# Finite S5 qualification plan

<!-- EVIDENCE:GUI-PERFORMANCE-IMPLEMENTATION:S5-PLAN-READY -->
Planning only, based on closure commit 12624fd2 and unchanged corrected R4 GUI
source c3949e53. No execution, production edits, builds or measurements are
approved. S4 stays complete. This plan replaces the obsolete readiness/remaining
work interpretation in the historical independent-replay packet, not its raw
results, required recipes or valid evidence.

## Decision and smallest path

Do not launch a final campaign yet. Two method boundaries prevent complete S5
acceptance: lifetime-complete DRM accounting is unproven, and the pinned native
Wayland adapter cannot deliver the required fractional LineDelta recipes.
Instantaneous resource/observer conformance also remains partial. A renderer
optimization or an unchanged benchmark cannot resolve these prerequisites.

The finite path is: (1) resolve the specific method feasibility decisions below,
(2) if they succeed, pin one candidate and one exact applicability/run manifest,
(3) run one consolidated independent qualification campaign, (4) report every
result and request separate owner product disposition. A failure ends the
affected batch; no repair/retry campaign is implicitly included. Method failure
ends the proposed full-qualification path with an explicit incomplete result.
Partial unaffected proof may be reported, but cannot become complete S5 acceptance.

The immediate recommended next authorization is **source-only method planning**
for M1–M3, with one deliverable per method and no native runs. It is not an
observer implementation or dependency adoption grant. If a sufficient method
cannot be specified, stop with that technical blocker rather than propose a
measurement campaign known to be incapable of qualifying it.

## Evidence credited and exact remaining scope

`inventory.json` preserves all 225 original row IDs, their controlling references,
consumer applicability, acceptance rules, historical status and existing evidence
references. It overlays PM051 withdrawal and existing exclusions explicitly;
historical 4/8 ms strings do not regain authority. It also checks all 46 existing
completed-group evidence hashes, all matching, and the S4 reconciliation's 17
hashes. Counts of rows are not counts of tests or failures.

- S4 checklist C01–C12, all 50 carried implementation rows, exact eight-sample
  R4/painter/clip/invalidation proof, corrected ten-entry native ledger and
  accepted pointer output/input remain credited. Do not repeat unit/GPU suites
  simply because S5 starts. Production source still matches the accepted candidate.
- The 46 completed groups include Console reconciliation, native focus/control
  and pane correctness, allocation observer controls, private realloc accounting,
  native 60-minute/600-cycle delivery and 20-recovery correctness. Their candidate,
  configuration and scope limits remain attached. A completed group is not a
  complete parent acceptance row.
- Accepted pointer trials supply exact output, visible crosshair, input lineage,
  normal drain and per-trial GPU tails at X11/1x. They do not supply clean total
  instrumentation effects, complete duty or all-config resource qualification.
  No pointer repeat merely to rediscover those same facts.
- The old Console mismatch and native three-slot ledger panic are superseded by
  passing correction proof. Historical failures remain archived. Retired partial
  redraw evidence is historical only; it is not proof of current R4 behavior.

| Remaining acceptance gap | Controlling requirement | Credit and smallest additional proof |
|---|---|---|
| Per-consumer positive/negative adoption, input/focus, clip/hit and final state | IC S5 and S1/S2/S3 qualification exits; SE E01–E09; individual 225-row map | Preserve 30 S1, 50 S2 and 20 S3 carried qualification predicates. Use existing negatives and source continuity; execute only missing native/configuration variants and independent replay. Cover actual 14 consumers, not invented absent schematic features |
| Admission, cold/churn/simultaneous-host resource bounds | IC GPI-S5; AM ADM-01, MEM-02, ACC-01/02; PM047 | Existing counts, guards and native compact lifetime delivery are partial. Need complete admission and simultaneous instantaneous/peak evidence on admitted configurations, including cold and recovery; refusal detecting an overrun is a failure, not a pass |
| Complete application duty and DRM resource lifetimes | AM MET-04, ACC-03; approved S4/S5 boundary | Own queue timestamps and held-fd endpoints do not cover disappearing clients/context retirement. M1 is prerequisite; no numeric duty pass available |
| Measurement validity and diagnostics effects | AM GPU-01–03, MET-02/04/06, ACC-01–03, STAT-01 | Credit R4 complete final-copy boundaries and 31/32-query proof. Timestamp-mode CPU increment is measured at pointer scope only. M2 must cover dropped/lost events, total observer incidence and matched off/on output; no universal subtraction |
| Native fractional wheel/scroll | AM W-ZOOM, W-SCROLL, W-MIXED; SE E04; AC-07 | Native Wayland source limitation is established; ordinary pixel input or synthetic WindowEvent cannot pass line predicates. M3 required; retain dat-wayland-fractional-line-proof-5bot |
| Retained CPU/resource limits and unexplained regression review | AM MET-03/05/06, STAT-01, MEM-02/03 | Historical pointer/clamp CPU and recovery CPU failures remain at old pins. Current R4 clean whole-workload numerical qualification absent. No current code defect inferred from old failures, no automatic CPU optimization. Warm-window success supersedes only its matching old scope |
| Stable-candidate endurance and recovery qualification | AM MEM-03 and REC matrix; IC S5 | Reuse producer hour/20 recoveries for established output and delivery. Their diagnostic timings and sampled memory are not full qualification. One independent combined stable-candidate hour can cover final replay and missing qualified metrics; no second producer hour |
| Independent replay and separate owner UX/product acceptance | PM045 GP-045-05; IC S5; END-02 | Existing independent design/source review is not native replay. One distinct reviewer may supply both missing coverage and required reruns; label these roles. Owner disposition follows concrete results |

IC = `specs/GUI_PERFORMANCE_IMPLEMENTATION_CONTRACT.md`; AM =
`specs/GUI_PERFORMANCE_ACCEPTANCE_MATRIX.md`; SE =
`specs/GUI_SHARED_ENGINEERING_CONTRACT.md`. Inventory retains exact per-row sources.

## Method feasibility and bounded stopping decisions

| Method | What is established | Unresolved postcondition / finite next deliverable |
|---|---|---|
| M1 DRM lifetime and retirement | Existing collector deduplicates known device/client identities; missing identity controls pass. Source review shows app hooks miss clients inside backend calls. Inspected wait/throttle APIs do not establish final context runtime transfer | One source-backed method specification covering open/dup/replace/close/exec/exit, process/client epochs, explicit context retirement, final counter availability, loss detection and deployed-driver identity. Name required privilege and overhead controls. If any mandatory lifetime or final accounting transition lacks a justified observation, mark infeasible/unproven and stop; no endpoint rerun, observer build or kernel changes |
| M2 resource and observer conformance | Stable allocation IDs, private-call/realloc overlap controls, five-image admission, bounded native compact delivery and query boundaries exist | One coverage table mapping ACC-01/02/03 and GPU-01–03 to existing events, missing opaque/driver/instantaneous incidence, storage limits and off/on controls. Distinguish application memory, driver residency and observer memory. If complete incidence or an unperturbed comparison cannot be demonstrated, affected numerical qualification remains unavailable. Do not design another general streaming system by default |
| M3 fractional native line input | Historical source review pins winit 0.30.13/SCTK 0.19.2 integer-only LineDelta and v7 binding. Pixel input is a different recipe | Recheck installed/locked source pins read-only, then choose a supported real input route or present an exact dependency/license/device-access proposal. Existing proposed v8/value120 patches are not approved. PM029 governs adoption. If no authorized native path exists, stop line-dependent cells; no synthetic substitution, rounding or permission changes |
| M4 native configuration and output oracles | Prior X11 controls/panes/window output and Wayland entry proof exist; portal consent previously timed out | Before execution, pin actual backend/60Hz/accepted dimensions/scale, fixture, camera/layout/selection/hover/pointer state, independent archived output references, engine and binary hashes. Unsupported display combinations stay explicit. Setup errors produce no measurements; incorrect input/output or incomplete drain ends the measured batch |

Sources: `implementation/S5/independent-replay/drm-client-method.json`,
`gpu-redraw-proposal/drm-finalization-source-review/receipt.json` and
`wait-api-followup.json`, `implementation/S5/private-allocation-lifetimes/result.json`,
`implementation/S5/independent-replay/packet.json#fractional_line_owner_boundary`.
Paths are relative to `docs/reviews/gui-performance/`. These are archived source
findings, not a claim that every possible backend method is impossible.

## Required repetitions and finite execution envelope (conditional, not approved)

Six configuration cells: native Wayland and X11/Xwayland, each at 1x/1.5x/2x and
60 Hz where supported. AM workload definitions govern applicability, including
T0/T1 F-DOA, one/two/four leaves, terminal visible/hidden and real auxiliary hosts.
Do not multiply inapplicable variants or silently collapse applicable ones.
The per-row consumer list and applicability are preserved in the inventory.

| Recipe | Fixed required count per applicable cell/variant, per necessary measurement mode |
|---|---|
| W-IDLE | Three independent 60-second trials |
| W-POINTER, W-ZOOM, W-PAN, W-PANES | Three 30-second trials, each five-second warmup and prescribed tail; exact input recipes retained |
| W-CLAMP | Three 30-second trials at each bound; separately timed inward-reversal control |
| W-SCROLL | Each Preferences host, pixel and line recipes separately, interior and both boundaries: three 30-second trials for each applicable combination; retain exact sign/scale conversion |
| W-CONTROLS | Three 30-cycle trials for each Preferences host: 90 cycles per host, 180 total |
| W-WINDOWS | Three 30-cycle trials for each of three auxiliary hosts: 90 per host, 270 total |
| W-LIFECYCLE | Three 20-sequence trials, retaining each applicable REC fault and supported native transition |
| W-MIXED | Three trials for each required zoom/scroll variant, with the exact 64 KiB/s stream, 4 KiB bursts and one-second 1 MiB burst |
| Cold startup/first use | Ten independent process starts for each applicable cold configuration; do not pool with warm observations |
| MEM-03 endurance | One 60-minute interval per admitted representative/demanding tier and applicable configuration, after five-minute warmup; at least 200 total window cycles and 20 injections; five-minute samples and first/last ten-minute comparisons |
| Formal relative claim | None proposed. If separately requested: exactly seven alternating AB/BA valid pairs, maximum ten attempted pairs, STAT-01 estimator; inconclusive ends study |

MET-01 repetitions apply to cycle recipes too. MET-06 requires diagnostics-off
resource acceptance and separately matched diagnostics-on structural trials,
with matching delivered output; do not add a third mode just to rename a metric.
A full three-trial absolute qualification is not replaceable by seven-pair inference.
Retain prescribed baseline ordering where comparison is required; no whole-project
improvement claim from a later narrow baseline. Report per-trial p95/p99/N/max,
counts, variability, raw failures and method limitations; do not pool away failures.

Pack warm recipes into existing cold-start processes after state restoration.
The three 30-cycle blocks for each auxiliary host produce 270 total cycles and
may satisfy the endurance 200-cycle minimum in the same qualifying hour. Pack
20 recovery injections and compatible lifecycle/mixed checks into that hour
with isolated exact windows. Existing runner's 290 slots are a packing aid, not
proof of all recipes or authorization to replace required three repetitions.
Do not add another hour solely for independent replay: the independent reviewer
executes the consolidated final campaign. If blocks do not fit without altered
cadence, retain the prescribed hour and schedule remaining blocks separately.

This defines required observation counts, not a fabricated fixed process total.
No native launch is presently scheduled. Exact launches cannot responsibly be
fixed until M1–M4 determine which cells/modes can be measured and existing evidence
can be credited. Before execution, freeze a finite manifest listing every
configuration/consumer/recipe/mode/repetition and credited omissions. For this
proposal, allow one attempt per declared absolute trial and one attempt per
missing deterministic negative control; a measured failure ends the batch, not
an automatic replacement. No extra trial to obtain a passing percentile.

## Stopping and interpretation

1. Source-only feasibility ends after the M1–M3 deliverables. An unresolved required
   postcondition blocks full qualification; it is not a renderer defect or a
   hardware impossibility conclusion. Further implementation needs its own scope.
2. At preflight, missing artifact/environment is a setup failure, not a timing
   result. Correct only within an approved setup scope; no GPU sampling until
   all prerequisites match. Preserve failed setup records.
3. Stop the affected measured batch for wrong native input/output, lost counters,
   observer overflow, incomplete submission/lifetime/drain, unsupported required
   capability, resource overrun or valid non-withdrawn limit failure. Preserve
   all observations. No golden relaxation, replacement sampling or optimization.
4. GPU 4/8 ms exceedance alone is never failure. Duty and CPU/resource limits
   remain unchanged pending their separate provenance review; diagnostics-contaminated
   numbers are not falsely promoted to clean acceptance or code-defect findings.
5. If final evidence is incomplete, report exact unqualified rows. No S5 closure
   without required correctness, valid numerical proof, independent replay and
   separate owner disposition. S4 remains closed unless a new actual defect is shown.

## Separate GBB-P05 and exclusions

GBB-P05 owns representative baseline/budget derivation, seven workload families,
total instrumentation effects, displayed responsiveness, supported-hardware
expectations and budget rationale. No new baseline collection or budget proposal
is scheduled by this S5 plan. Overlapping resource/method evidence may be reused
in both lanes; that does not create two campaigns. Displayed latency/pacing,
resize resource/temporal investigation, T2 and unadmitted schematic performance
remain explicitly outside initial S5 acceptance under AM initial-scope and IC
resize amendments. Static output, input/focus and admitted resource correctness
remain required. No concurrent multi-device qualification is inferred from serial
shared-device proof. Functionality remains globally unblocked.

<!-- OWNER:GUI-PERFORMANCE-IMPLEMENTATION:GPI-S5-PLAN-DISPOSITION:PLAN -->
Recommended disposition: authorize only source-only M1–M3 feasibility planning,
or defer S5. Do not approve a final native campaign yet. A concrete, complete
method and frozen finite run manifest must precede any execution request.
