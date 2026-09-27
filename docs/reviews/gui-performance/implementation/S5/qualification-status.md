# S5 completed tests and remaining acceptance

Current sequencing: the 2026-09-27 owner amendment reopens S4 for GPU redraw
architecture correction and makes S5 pending behind that exit. Current work is
solution discovery/specification only; implementation, builds and runtime
experiments require approval of a concrete proposal. See
[`gpu-redraw-s4-owner-direction.json`](../../gpu-redraw-s4-owner-direction.json)
and the amended implementation contract. Every result below remains credited
only at its recorded candidate/scope; no prior test is reset or new pass claimed.

This checklist records existing validated evidence from `implementation-map.json`.
**An agent handoff does not reset a completed test.** Done applies to the exact
test scope and candidate in the linked evidence; it does not turn a partial
requirement into full S5 acceptance. Ignored tests are not passes. Test-group
counts overlap and must not be summed. No tests were rerun to create this list.

## Done — carry forward

- [x] **Console regression tests** — 9 passed; 0 failed. [Evidence](console-reconciliation/result.json)
  Scope: Console behavior on recorded candidate.
- [x] **Console static goldens** — 10 scale cases across seven manifests; exact zero-pixel differences. [Evidence](console-reconciliation/result.json)
  Scope: Named Console corpus; broader visual rows remain separate.
- [x] **Canonical native shell screenshot** — 0.320% differing pixels and0.2914 mean channel delta; both within existing limits. [Evidence](native-parity/result.json)
  Scope: Recorded1680x1050 fixture and backend; not every native configuration.
- [x] **Reference and custom-icon gates** — Board-editor reference accepted;11custom icon checks pass. [Evidence](native-parity/result.json)
  Scope: Recorded static references, not full product parity.
- [x] **Renderer unit regression suite** — 353passed;89ignored were not executed. [Evidence](resource-snapshots/result.json)
  Scope: Completed serial visual-feature suite; ignored tests not counted as passed.
- [x] **Application unit regression suite** — 400passed;17ignored;0failed. [Evidence](native-owner-focus/verification.json)
  Scope: Completed application suite on focus-fix candidate; ignored tests not counted.
- [x] **Serial GPU regression group** — 33serial GPU tests pass. [Evidence](resource-snapshots/result.json)
  Scope: Named existing group; this does not erase separately tracked concurrent-GPU failure. [Affected follow-up](lazy-terminal-pipeline/result.json): one terminal-image GPU test passes after deferring unused pipeline compilation; empty/first-use/reuse/same-device replacement checked. Native CPU benefit remains unmeasured. [Shared glyph follow-up](lazy-terminal-pipeline/shared-glyph-result.json): layered workspace/menu pixels match independent pipelines; cache retry/reindex and same-device replacement pass in one affected test.
- [x] **Missing-final-resolve negative and restored positive** — Injected missing resolve fails pixel oracle; restored candidate passes1/1. [Evidence](final-msaa-resolve/result.json)
  Scope: Final-stage ordering/resolve sensitivity, unchanged8xquality; not absoluteGPU budget.
- [x] **Native menu output regression** — Six normal runs;60menu cycles; exact baseline pixel matches. [Evidence](final-msaa-resolve/result.json)
  Scope: Bounded native menu replay, not complete W-WINDOWS or performance qualification.
- [x] **X11 auxiliary focus positives and negatives** — All3missing-owner-hint negatives fail focus; all3production positives restore focus;6exact captures and normal exits. [Evidence](native-owner-focus/result.json)
  Scope: Global/Project/New X11/1x owner-focus behavior.
- [x] **Wayland native-entry close regression** — 1test passes;12host/profile close cases. [Evidence](native-owner-focus/verification.json)
  Scope: Direct native-handler invocation, not OS delivery; constrained2xheight limitation retained.
- [x] **Global Preferences30-cycle controls sequence** — 30cycles;510actions;540image checks;90drag/cancel motion receipts. [Evidence](controls-30-cycle-batch/result.json)
  Scope: Actual X11/1x host functional sequence; numerical/full configuration/negative matrix still separate.
- [x] **Project Preferences30-cycle controls sequence** — 30cycles;510actions;540image checks;90drag/cancel motion receipts. [Evidence](controls-30-cycle-batch/result.json)
  Scope: Actual X11/1x host functional sequence; completed Project sequence was not rerun for supplemental clip probe.
- [x] **Hidden-control/Search overlap probes** — 30actual overlap probes per host preserve fullclient pixels and do not activate hidden control. [Evidence](controls-30-cycle-batch/result.json)
  Scope: 60supplemental native probes; original inert-header-only evidence retained separately.
- [x] **Native pane transition sequence** — 60actions/15cycles over30seconds;PaneId/camera retention;exact Board interior;normal drain. [Evidence](pane-native-sequence/review.json)
  Scope: One X11/1x functional sequence; not three-trial/all-scale performance.
- [x] **Pane tracked-allocation lifetime reconciliation** — 606registrations/606releases;25040events;341snapshots;final tracked capacity0. [Evidence](pane-resource-sequence/review.json)
  Scope: Tracked application allocations only; not complete driver/private/API-instant peaks or endurance.
- [x] **Private-call observer controls** — 2explicit serial observer tests pass, including transient peaks,overlap,overflow and abandoned calls. [Evidence](private-call-observation/result.json)
  Scope: Observer conformance, not full private-memory qualification.
- [x] **Private-call native delivery and refusal controls** — 6373calls/12746events reconcile;native overflow/existing-file controls also retained. [Evidence](private-call-observation/result.json)
  Scope: Recorded9cycle run;66historical null measurement origins retained, addressed by separate origin test.
- [x] **GPU allocation observer controls** — 2explicit serial observer tests pass. [Evidence](gpu-allocation-observation/result.json)
  Scope: Identity,release,shared-reservation peak and overflow delivery conformance.
- [x] **GPU allocation native delivery** — 6961events;184allocations/releases;no unattributed origin;malformed-history controls rejected. [Evidence](gpu-allocation-observation/result.json)
  Scope: Tracked-event delivery, not complete GPU residency or numerical caps.
- [x] **Four-host measurement-origin/refusal/retry test** — 1explicit GPU test passes all4actual adapters;all measurement calls attributed;zero dropped events. [Evidence](measurement-origin/result.json)
  Scope: Direct adapter proof; original native null-origin evidence remains historical.
- [x] **Resource observer replacement/retirement conformance** — RealGPU observer test passes shared budget IDs,retirement and metadata-only ownership. [Evidence](resource-snapshots/result.json)
  Scope: Observer semantics; full instantaneous accounting remains outstanding.
- [x] **Endurance harness method and cleanup review** — Independent method findings corrected and verified. [Evidence](independent-replay/method-review.json)
  Scope: Method review only. Both incomplete native attempts remain failed; no endurance pass.
- [x] **Fractional pointer offline controls** — Schedule plus6simulated portal cases pass; independent source review verified corrections. [Evidence](independent-replay/pointer-method-preparation.json)
  Scope: No native pointer stream executed; no delivered-input or performance pass.

- [x] **Lazy world pipeline initialization and pixels** — Focused hardware test passes at 4× and 8× MSAA: overlay omission, cold/warm board output and renderer replacement. [Evidence](lazy-world-pipelines/result.json)
  Scope: Exact eager/lazy pixel comparison on one GPU; native performance and full recovery remain open.

- [x] **Glyph preparation count lifecycle** — One hardware test passes populated/empty counts, reuse, cancellation and replacement. [Evidence](glyph-preparation-counts/result.json)
  Scope: Last-successful preparation groups only; full ADM-01 and native delivery remain open.

- [x] **Text admission reuse and unique-key controls** — One hardware test passes exact observer pixel parity, duplicate-key counts, reuse and invalid-input recovery. [Evidence](text-admission-observation/result.json)
  Scope: Sampled production-shaped-buffer observation; full frame/pane/native admission remains open.

- [x] **Prepared world geometry and source identity** — Three tests pass range/fallback validation, missing-data rejection, triangle boundaries and immutable source identity. [Evidence](geometry-admission/result.json)
  Scope: Read-only prepared-world observations; native delivery and full admission remain open.

- [x] **Render-attempt observer and trace delivery controls** — Two hardware tests pass pixel parity/reuse, delivery and overflow rejection. [Evidence](frame-admission-delivery/result.json)
  Scope: Offscreen render/writer conformance; native workloads, overhead and full admission remain open. Actual composed-world encoded ranges, pane identity and absent-data controls also pass. [Encoded-world supplement](frame-admission-delivery/encoded-world/result.json) Screen/grid delivery and terminal geometry controls also pass. [Immediate-geometry supplement](frame-admission-delivery/immediate-geometry/result.json) Pane text-origin composition, pixel/cache parity and JSON reconciliation pass. [Text-origin supplement](frame-admission-delivery/text-origins/result.json) Actual screen/grid submission and omission controls pass. [Screen-submission supplement](frame-admission-delivery/screen-submission/result.json) Fourteen offline analyzer controls pass. [Analyzer integration](frame-admission-delivery/analysis-integration/result.json) Terminal session reorder/close attribution passes its CPU composition control. [Terminal-origin supplement](frame-admission-delivery/terminal-origins/result.json)

- [x] **X11/1x 60-minute auxiliary window cycles and resource trace reconciliation** — 600 timed cycles (200 per host), all603 cycles reached exact pixel readiness/focus/closure, normal drained exit0;604 renderer lifetimes reconciled and no sampled cap violations. [Evidence](window-endurance/native-60minute-4d645582/assessment.json)
  Scope: This producer run and delivered traces;288 intermediate nonmatching readiness captures are retained. [Saved-run drift analysis](window-endurance/native-60minute-4d645582/drift-analysis.json) excludes over5% diagnostics-on CPU regression; sampled RSS grows3,112,960bytes and New Project readiness mean/p95 grow6.18%/6.70%. These remain unqualified for native latency/resource-trend acceptance. Recovery injection, broader coverage, overhead and full MEM-03 remain open.

- [x] **Twenty native backend device-loss recoveries, X11/1x** — Five each for Main, Global Preferences, Project Preferences and New Project; exact selected-window pixels, focus and workspace/registry receipts. Normal shutdown; 51 renderer origins, 719 GPU allocations and 40,648 private calls reconciled. [Evidence](repeated-device-recovery/native20-ba24e35a/assessment.json)
  Scope: Producer structural recovery on ba24e35a; all 20 diagnostic CPU intervals exceed 100 ms. Full performance, simultaneous four-host/mixed-PTY and backend/scale acceptance remain open. Two driver failures are preserved. This run is separate from the earlier hour.

- [x] **X11/1x warm-window CPU and native output, three trials per host** — All270candidate opens≤50ms (maximum44.440ms) and closes≤20ms (maximum11.952ms); exact pixels/focus/destruction. Six fixed baseline/candidate runs,558first-use/warm cycles, normal exits and display restoration. [Evidence](lazy-terminal-pipeline/native-startup-batch/assessment.json)
  Scope: Candidate0709fc90, actualGlobal/Project/New; baseline also passes this batch. Six intermediate nonmatching readbacks and historical failures retained. No causal speedup, otherbackend/scale, GPU/full resource, independent replay or owner UX acceptance.

- [x] **Unused raw-device subscription removal: X11/1x clamp behavior** — Three trials, both bounds; all18reference images exact, inward reversal and normal cleanup preserved. CPU1.14–1.21% exceeds1% in all six intervals; this check does not mark CPU or full W-CLAMP accepted. [Evidence](raw-device-input/result.json)

- [x] **Auxiliary close-after-own-submit callback lifetime** — One six-case X11 run covers Global/Project/New, each with normal and deliberately strong-capturing callbacks. Native close, shared retention oracle, healthy post-release dispatch and ledger retirement pass. Full driver-memory/close-budget and LF-07 acceptance remain open. [Evidence](close-after-submit/result.json)

- [x] **Terminal idle-poll regression batch** — 19 passed; wake consumption, deferred/no-proxy polling, bounded draining and close behavior. Native CPU cap still fails; relative performance effect remains unqualified. [Evidence](terminal-idle-poll/result.json)
- [x] **Private redraw admission negative** — Distinct reviewer rejects non-applied NEW-adapter bypass of shared frame admission/completion. Source/admission proof only. [Evidence](END-04/result.json)

- [x] **CPU cache retention across device replacement** — Extended existing GPU test passes with a distinct device, retained CPU identities/reuse, cross-host rejection and fresh-GPU pixel parity. App integration compiles; native recovery timing/resource acceptance remains open. [Evidence](device-cpu-cache-retention/result.json)

- [x] Camera route layout reuse: one guarded batch of17existing camera/pane/grid helper tests passed. Pointer routing builds one layout instead of two by source inspection. Native routing and CPU-cap acceptance remain open; one layout allocation remains. See `camera-route-layout/result.json`.

- [x] Monitored private-call realloc peak/refusal:13allocator/call-guard controls passed in the final batch;1existingignored. Initial passing batch retained; repeated only after scope correction. Explicit replacement overlap reaches the guard. Forced-copy cost, native caps and retained/scratch classification remain unqualified. See `scoped-reallocation-peak/result.json`.

- [x] Active-call block transition delivery:4serial observer controls pass and app writer compiles. Allocation/free order, realloc overlap, summary compatibility and overflow covered. Three offline classifier controls and the recorded compact X11/1x native integration pass at their stated scope; complete semantic accounting, overhead and full-scope qualification remain pending. See `private-allocation-lifetimes/result.json`.

- [x] **Unchanged per-pane grid geometry reuse** — 5 renderer and17 viewport grid-filter tests plus application check passed. Admitted cache preserves output, invalidates exact dependencies and releases refused/empty storage. No native performance-cap claim. [Evidence](grid-reuse/result.json)

- [x] **Merged-pass GPU correctness and accounting controls** — 35 passed initially; omitted retained-grid assertion corrected and the one affected test passed. Source review and renderer check passed. Candidate4eaa0087; exercised GPU layers/resources/timestamps. Not full native or numerical acceptance. [Evidence](pass-consolidation/result.json)
- [x] **Merged-pass native output and clean shutdown** — Six fixed native runs: exact reference pixels, stable pins, normal exits and clean display/process restoration. Candidate4eaa0087 X11/1x. All three quiet CPU and all three GPU p95/p99 trials failed; only output/lifecycle correctness is done. [Evidence](pass-consolidation/native-performance/assessment.json)
- [x] **Retained triangle compaction bounded correctness** — One allocation/range/refusal unit and one exact GPU parity test passed at8xAA/scales1and1.5, with warm reuse and missing-triangle negative. Candidate9f16eb69. Board GPU fixture and shared converter; no full schematic/native or performance acceptance. [Evidence](triangle-compaction/result.json)
- [x] **DRM missing-device identity controls** — Four offline controls passed; missing/empty devices remain unidentified rather than falsely merged. Candidate2d57a230 collector. Endpoint parser/grouping only; client lifecycle, drain and observer overhead remain unqualified. [Evidence](independent-replay/drm-client-method.json)

## Still open — do not reset the done list

Private allocation startup delivery now passes one compact-record X11/1x integration run:201547events/3025calls, three exact auxiliary pixels, three live-device DRM endpoints and normal exit. Four producer controls and three classifier controls pass. Both earlier startup failures remain preserved. See `private-allocation-lifetimes/native-integration/compact-result.json`. This closes the observed burst-delivery defect, not native matrix, overhead, full accounting or independent acceptance.

- [ ] Resolve recorded pointer CPU/GPU and clamp CPU failures. Current X11/1x warm-window CPU/output qualification is checked above; other configurations and final independent replay remain. Preserve historical Project-open excesses.
- [ ] Complete missing native backend/scale, input, semantic-output and required negative cases; reuse already completed host sequences. The [current call-site inventory](END-03/result.json) reuses S1–S4 ownership proof; the concrete private redraw admission negative is now rejected. The auxiliary callback/strong-capture gap is checked above. The END-04 NEW registration declaration is reviewed; full positive qualification remains incomplete.
- [ ] Complete fixture admission and instantaneous private/scratch/driver/client resource accounting, including observer overhead.
- [ ] Resolve the saved endurance resource trend and missing in-scope tier/configuration, accounting and recovery-performance proof. The hour and separate 20 structural recoveries above are done. Readback timing remains descriptive; deferred displayed latency is not an initial S5 gate. Earlier failed attempts remain failures.
- [ ] Complete independent actual replay against the stable final candidate, followed by separate owner UX/product acceptance.
- [ ] Resolve applicable recorded drift failures; preserve separately tracked concurrency and golden failures.

S0–S4 implementation exits remain complete. Their carried qualification is part
of the outstanding original predicates, not a reason to repeat implementation.
Pinned resize/temporal,T2 and unadmitted schematic exclusions remain unchanged.

## Remaining execution without duplicate campaigns

Historical CPU check: candidate `5b013adb` ran three fixed quiet clamp trials; all six
bound intervals exceed the 1% ceiling (**1.047673%–1.122825%**). Endpoint, reversal,
focus and clean shutdown checks passed. This is a recorded CPU failure, not a
completed acceptance check. [Evidence](terminal-idle-poll/native-clamp-batch/assessment.json)

All 46 completed-group evidence hashes match. The original map contains 225
requirement rows, not 225 separate tests: 15 specification-record rows, 14
nonblocking resize rows, eight wholly deferred temporal rows, and 188 initial
or mixed-scope rows. HP05-02 still requires idle resources; its pacing component
is deferred. A row count is not a remaining trial count.

Use existing source and negative-control proofs first. Resolve demonstrated
performance defects and measurement gaps before final native qualification.
The existing map now lists six final backend/scale campaigns: Wayland and
X11/Xwayland, each at 1x, 1.5x and 2x where supported. Each contains the applicable
workload/host/pane/terminal variants and prescribed repetitions; six campaigns
must never be reported as six tests. Exact process launches depend on workload packing and remaining methods;
a fixed global launch total is not an additional acceptance gate. Each expensive
batch still needs complete applicable variants, fixed repetitions and a stopping
condition. No new native campaign is scheduled by this edit.

The distinct reviewer's contract reading confirms that one final independent
campaign may supply missing qualification and independently rerun producer
trials. No second blanket producer final campaign is required. New coverage
and reproduced results must remain distinguishable. MET-06 still requires
separate diagnostics-off resource trials and matched diagnostics-on structural
trials. Endurance, negative controls, baseline ordering and owner UX acceptance
remain unchanged. See the [existing replay packet](independent-replay/packet.json).

Before another run, name the original requirement, decision enabled, missing or
invalidated evidence, fixed repetitions and stopping condition. Preserve failed
runs; investigate cause before retry. Existing evidence may satisfy multiple
predicates with distinct assertions, but one trial cannot count as several
required repetitions. Seven-pair relative inference is required only when making
that claim; it is not an extra condition for every absolute budget result.

Pointer input clarification: [reconciliation of all seven saved runs](pointer-native-batch/input-reconciliation.json)
accounts for all 3,600 scheduled requests: 2,320 repeat the current rounded
integer coordinate and all 1,280 changed coordinates reach native receipts in
order. W-POINTER does not impose an additional 120 **distinct** native positions/s
condition. No new fractional-pointer run is required merely to satisfy that
invented condition. Semantic hit/crosshair correctness, acknowledged semantic
actions, numerical failures and actual fractional wheel/scroll requirements
remain open. No native run was repeated for this reconciliation.

Independent contract review also confirms that MEM-03 requires **200 window
cycles total**, not 200 per host. The existing 600-cycle producer result stays
done. Three 30-cycle W-WINDOWS trials for each of the three hosts yield 270
cycles that can also contribute to the same qualifying endurance hour. Required
recovery/lifecycle and mixed work can share that hour with distinct measurement
windows. Cycle recipes still require three repetitions; one block cannot count
three times. The hour is per admitted tier, not automatically three hours.

Cold-start processes can continue into warm trials after first-use recording and
state restoration; separate launches per metric or host are unnecessary. Keep
independent repetitions, baseline ordering and diagnostics-off/on modes intact.
These are scheduling reductions, not native acceptance or a complete runner.
See the existing map's `process_and_endurance_reuse` review.

Earlier CPU check: candidate `01b99830` / binary `8eff3ed55ae2` completed three combined quiet X11/1x runs. Pointer **10.445–11.288%** exceeds10%; all six clamp intervals **1.125–1.246%** exceed1%. Exact pointer endpoints, unchanged clamp pixels, inward reversals, pinned inputs and normal exits/display restoration passed. No numerical acceptance; no unchanged rerun. [Evidence](grid-reuse/native-cpu/assessment.json)

- [x] Retired partial-redraw candidate `500387c1`: seven focused correctness tests passed, including exact GPU mixed-layer/pane/hover output and failure controls. These remain historical passes; native performance failed and the implementation was retired. [Correctness evidence](interaction-damage/result.json)

The fixed six-run pointer assessment failed: quiet CPU **15.378–16.361%** (limit10%); GPU **15.002–15.132ms p95 /16.847–17.172ms p99** (limits4/8ms). All six runs exited normally with exact endpoints; all three GPU runs received1280/1280changed pointer positions. Restore the prior production path, retain all failures and do not repeat this rejected candidate. No formal relative-effect claim from unpaired historical comparisons; no full GPU-duty/client-lifetime acceptance. [Assessment](interaction-damage/native-performance/assessment.json)

Latest measured candidate `4eaa0087` still fails pointer CPU **10.725–11.032%** and GPU **12.026–12.073ms p95 /12.438–12.528ms p99**. Triangle compaction has bounded correctness proof but no later native performance result. Do not repeat unchanged qualification; retain clamp failures and all broader acceptance gaps. [Latest assessment](pass-consolidation/native-performance/assessment.json)
