# Shared GUI performance recovery: specification-development plan

Status: active planning contract; renderer mechanisms and numeric budgets are not
ratified.

On 2026-09-19 the owner directed completion of GPS-C02 through GPS-C06.
`docs/reviews/gui-performance/specification-resumption-owner-direction.json`
records the exact instruction. Specification development now precedes the
pending GPS-C01R recovery. Resize CPU and content blinking remain unresolved;
neither this ordering nor the eventual specification decision accepts runtime
performance. GPS-C06 still requires the owner's decision on the completed,
independently reviewed packet. Preserve all R01–R42 and HP01–HP25 obligations.

## Authority and owner disposition

On 2026-09-17 the owner approved implementing the roadmap amendment only:
register specification development as the next planned priority, preserve the
other session's bounded Preferences QA, and block rendering implementation
behind specification approval and a separate execution grant. The owner chose
shared-foundation scope, qualification on the daily Linux setup first, a 60 Hz
baseline, and a narrow permanent exception for read-only research.

This document governs specification development. The subsequent owner sequencing
direction authorizes bounded baseline measurements using existing tooling first,
and narrowly scoped diagnostic instrumentation only for otherwise unavailable
measurements. Product changes follow a written engineering contract and slice
proof definition; specification approval and final implementation acceptance
remain distinct. No completion step is completed merely by registering this plan. Consult the generated
Active Frontier for current selection; this document does not maintain a rival
next-task answer.

Decision 025 governs roadmap/claim authority; decision 023 and the universal
viewport contract govern shared tooling; decision 029 governs dependencies.
Existing rendering, typography, selection, Preferences, and terminal decisions
remain controlling. Future cross-cutting mechanisms require the next available
numbered decision and explicit owner ratification. Do not reserve a decision
number now or describe proposed mechanisms as approved.

## Performance-first requirements for the future specification

Excellent visual rendering, excellent UX responsiveness, and low CPU/GPU/memory
consumption must be accepted together. Performance is a first-class architecture
requirement for every window, pane, and control, not a final optimization pass.
Eliminate demonstrated avoidable work even after timing ceilings pass. Never
trade dropped input, sluggish updates, stale content, inaccurate hit testing,
or reduced visual quality for lower utilization. The least practical resource
use is a measured engineering objective, not a claim of a provable global minimum.

Qualify scrolling, zooming, panning, hover, opening/closing, switching/splitting
panes, resizing/maximizing, DPI changes, terminal activity, and long sessions.
Distinguish cold startup, first-use preparation, warm interaction, and return
to idle. Cover all existing consumers; start native performance qualification
on the owner's daily Linux environment and keep other GPU/compositor coverage
explicitly unqualified until measured.

The read-only findings in the linked research source justify investigation of
invalidation, scheduling, retained preparation, GPU upload identity, shared
controls, and production-path tests. They do not prove current timing failure
or justify replacing Rust/wgpu. Reconcile later commits before reusing findings.

## Scheduled specification completion contract

The clause inventory binds the individual requirements below to the six existing
completion steps. Step state is maintained in the Frontier. This amendment alone establishes no
completed research, specification, runtime proof or product acceptance.

<!-- REQ:GUI-PERFORMANCE-SPEC:GPS-C01 -->
### Audit reconciliation

<!-- GUI-PERFORMANCE:R01 -->
**R01.** Pin the inspected commit, dirty-state boundary, build and tool versions.
Reconcile the audit source, recent QA fixes, every GUI consumer, cache lifetime, event
route, surface lifecycle and existing test. Read complete owning evidence routes.
Produce a code-linked consumer/finding inventory distinguishing confirmed paths,
suspected costs, measured costs, resolved findings and missing proof. Preserve
Preferences ownership disposition, acceptance history and independent engine work.

<!-- GUI-PERFORMANCE:R02 -->
**R02.** Require a reproducible baseline report before completing this step: current
optimized-build profiles of scrolling, zooming, panning, hover, Preferences, pane/window
opening and closing, switching, and idle on the daily Linux setup. Pin the candidate,
real project/fixture hashes, input schedules, environment, commands, raw observations
and repeat variability. Separate cold, warm and idle phases; distinguish physical/native
input from handler replay. Historical QA percentages and screenshots alone cannot
satisfy this baseline.
For imported fixtures, also pin resolved object identities and ordering: a raw
file hash alone is insufficient when the importer derives IDs from its path.
Use a frozen native fixture or the same controlled import identity on both sides.

<!-- GUI-PERFORMANCE:R03 -->
**R03.** The owner's subsequent sequencing direction grants bounded measurement
execution within GPS-C01: inspect existing diagnostics first; build the current
optimized GUI with the guarded runner; launch only session-owned instances on
copies of real fixtures with isolated configuration; collect CPU time, GPU engine
activity where available, memory, preparation/encoding/upload work and event/frame
observations for idle, pointer, camera, scrolling and pane/window transitions.
Retain exact revision/build/fixture/environment, input schedule, raw observations
and limitations. Cap initial samples at three 30-second trials per workload after
five-second warmup; final endurance qualification is a later acceptance activity.
Do not operate on the owner's live project or record generated state in tracked
fixtures. Existing native capture and OS counters take precedence over new code.

If a metric is unavailable, allow the minimum opt-in diagnostic counters or
controlled production-handler driver needed to measure it, with the exact source
scope, overhead and limitations recorded before use. Diagnostic execution is not
permission to change rendering semantics, adopt dependencies or run a rewrite.
Keep synthetic handler input distinct from native input and displayed feedback;
missing measurements cannot be reported as passes. No new approval framework or
historical acceptance refresh is required for these owner-authorized measurements.

<!-- GUI-PERFORMANCE:R04 -->
**R04.** Reconcile applicable prior art with Datum's existing shared tooling and
production paths. Publish an alternatives table tying each proposed change to a measured
cost or demonstrated correctness/lifecycle defect, expected benefit, complexity,
resource cost and a falsifiable verification method. Preserve working infrastructure
unless evidence supports replacement; external reference code is not dependency adoption
authority.

<!-- REQ:GUI-PERFORMANCE-SPEC:GPS-C02 -->
### Shared architecture specification

<!-- GUI-PERFORMANCE:R05 -->
**R05.** Define per-window scheduling and damage ownership, event-to-frame state
transitions, frame coalescing without input loss, backpressure and bounded pending work.
State which changes invalidate which consumers, when redraw requests are suppressed, and
how a continuously active terminal or pane avoids starving input and presentation
elsewhere. Specify interfaces and ownership rather than requiring a base-class
hierarchy.

<!-- GUI-PERFORMANCE:R06 -->
**R06.** Specify retained shell, pane, interaction and dialog preparation; text/geometry
cache lifetime, immutable resource identities, upload invalidation, eviction and
release. Define ownership across window closure and recovery, memory bounds and the
production counters that prove unchanged geometry is neither rebuilt nor uploaded.
Buffer capacity reuse and pointer/length identity alone are not proofs of valid retained
content.

<!-- GUI-PERFORMANCE:R07 -->
**R07.** Map every existing consumer to shared scheduling, scrolling, clipping and
controls or an explicit semantic exception. Preserve terminal screen authority, row
versus continuous scrolling, painter order, layer semantics, antialiasing, typography,
selection and the canonical design mutation path. State the extension contract future
panes must use; distinguish configuration from duplicated machinery.

<!-- GUI-PERFORMANCE:R08 -->
**R08.** Publish a lifecycle matrix covering zero-size, surface lost/outdated, timeout
and repeated timeout, device failure, allocation failure, occlusion/minimize/restore,
suspend/resume, DPI changes and moving windows between supported displays, plus window
closure during pending work. For each case define detection, state transition, bounded
retry/backoff, resource release/recreation, data preservation and recovery or controlled
shutdown with user-visible behavior. Unsupported platform signals require an explicit
applicability disposition.

<!-- GUI-PERFORMANCE:R09 -->
**R09.** For every lifecycle row specify a production-path fault/recovery test, expected
CPU/GPU activity and memory behavior, retry/time bounds, input/focus behavior and
evidence. Cover simultaneous window activity and repeated failure/recovery; no spin,
unbounded allocation, silent input loss or stale/hit-test-inconsistent frame may be
accepted as recovery. Distinguish deterministic fault injection from later native
platform qualification.

<!-- GUI-PERFORMANCE:R10 -->
**R10.** Keep architecture proposals provisional until the baseline and alternatives are
reconciled with the acceptance contract. This step may finish a reviewable draft; final
architecture approval requires the later workload/budget matrix, complete requirement
mapping and independent review. Record tradeoffs against measured work and visual/UX
requirements; a timing ceiling does not excuse demonstrated avoidable resource use.

<!-- REQ:GUI-PERFORMANCE-SPEC:GPS-C03 -->
### Performance, resource, visual, and UX acceptance contract

<!-- GUI-PERFORMANCE:R11 -->
**R11.** Define a reproducible workload matrix using named real projects and existing
valid fixtures, with content hashes, exact input schedules, duration and expected
visible output. Cover scrolling, zooming, panning, hover, opening/closing,
switching/splitting, resize/maximize, DPI, terminal activity and idle. Record OS/kernel,
adapter/driver/backend, compositor/display refresh, logical/physical size, scale, build
settings and measurement tool versions. Identify each required daily-Linux workload and
each additional unqualified environment.

<!-- GUI-PERFORMANCE:R12 -->
**R12.** Declare supported scale tiers with measured geometry, layer, text/glyph and
visible-object counts, pane/window counts and DPI. Include a representative demanding
real project and mixed activity such as terminal output during Preferences scrolling or
editor zoom. Specify bounded input bursts and saturation behavior. Small-fixture results
cannot qualify larger tiers; unavailable large fixtures remain a coverage gap requiring
resolution or explicit owner scope disposition.

<!-- GUI-PERFORMANCE:R13 -->
**R13.** For every workload/metric row specify units, numerator/denominator, observation
boundaries, capture method, sample count, percentile/max calculation, allowed noise,
numerical threshold and pass/fail rule. Pin raw data and scripts/commands. Separate CPU
work, upload copies, GPU execution, acquisition waits, presentation feedback and
input-to-display observations; submission counts are not delivered FPS, utilization is
not energy, and a timestamp before presentation is not proof of visible completion.

<!-- GUI-PERFORMANCE:R14 -->
**R14.** Provide a complete qualification matrix with measured, unsupported and
missing-evidence dispositions. A missing required metric is not a pass or a note-only
exception: require a validated alternate method with stated uncertainty, or an explicit
owner-approved qualification scope limit. Keep the affected environment/workload
unqualified; no blanket enterprise/performance-ready claim is allowed. Before
specification approval every required daily-Linux row needs a feasible measurement
method and numerical acceptance rule; runtime passes are future implementation evidence,
not fabricated specification outputs.

<!-- GUI-PERFORMANCE:R15 -->
**R15.** Define total process CPU-seconds per action and active/idle CPU duty limits,
normalized to one core and including all application threads. Account separately for
cooperating engine processes so work cannot disappear across a process boundary.
Distinguish on-CPU work from waits and frame latency. Cover input processing,
preparation, encoding and background work; lower renderer timings alone cannot establish
lower application resource consumption.

<!-- GUI-PERFORMANCE:R16 -->
**R16.** Define GPU execution/duty and upload-byte budgets, command/submission counts
and resource-allocation measurements with adapter/backend support limits. Measure
instrumentation overhead and avoid synchronous GPU readback waits in the normal render
path. When timestamps are unavailable use an independently validated alternative or
leave that coverage explicitly unqualified under the preceding rule. Do not infer GPU
efficiency from CPU improvement.

<!-- GUI-PERFORMANCE:R17 -->
**R17.** Define end-to-end input latency distributions and maximum allowed application
stalls for each interaction, including opening/closing and pane switching. Account for
the complete scheduled input stream, coalesced events, final state, expected display
opportunities, late/missed updates and return to idle; observing only successful frames
must not hide dropped input or updates. Bound queued work and terminal/input starvation.
Frame pacing, input correctness and visual fidelity must pass together.

<!-- GUI-PERFORMANCE:R18 -->
**R18.** Specify CPU-resident and GPU-resident resource accounting,
peak/steady-state/cache budgets, allocation churn and tolerances for recovery after
closing consumers. Define exact soak durations, cycle counts, input rates and
plateau/trend tests for representative and demanding tiers, including repeated
open/close and failure/recovery. Bound latency drift and return-to-idle time. Explain
allocator/driver retained capacity separately from live resources; a short sample or a
merely flat high allocation is insufficient proof of efficient long-session behavior.

<!-- GUI-PERFORMANCE:R19 -->
**R19.** Ratify defensible numbers only after reviewing baseline, refresh rate,
supported scale and measurement noise. Define warmup and cold-start handling,
independent repeats, outlier handling, regression comparison, statistical confidence and
repeat/reject rules. Retain raw failed trials; do not improve results by silently
dropping stalls or changing workload/output. The following conversational numbers are
review starting points, not ratified budgets or a sufficient acceptance suite.

| Candidate | Proposed review starting point |
|---|---|
| Routine CPU event/preparation/encoding, excluding presentation waits | p95 <=4 ms; p99 <=8 ms |
| Continuous native 60 Hz interaction | >=95% of observed presentation intervals <=20 ms; no unexplained application-caused warm stall >50 ms; complete input/opportunity accounting and latency limits above are additionally required |
| Quiet idle | <=1% of one CPU core over 60 seconds; no application-generated render/submission loop |
| Resources | Bounded caches; no monotonic retained-resource growth or progressive latency degradation; explicit peak, plateau and release budgets required |
| Comparative efficiency | Investigate repeatable CPU/GPU cost increases >5% under identical work/output before acceptance |
| Short-run sampling | Five-second warmup; three thirty-second optimized-build samples; separate debug/cold-start reports; does not replace endurance qualification |

<!-- GUI-PERFORMANCE:R20 -->
**R20.** Specify matching native visual and interaction evidence for each consumer:
clipping, hit testing, focus/keyboard navigation, applicable text input/IME and
accessibility behavior, typography, selection and non-color states. Preserve controlling
visual sources. Pair automated production-path checks with native owner UX review. Lower
utilization through dropped input, stale content, degraded rendering or sluggish updates
fails acceptance; resource ceilings do not end the search for demonstrated avoidable
work.

<!-- REQ:GUI-PERFORMANCE-SPEC:GPS-C04 -->
### Requirement-linked implementation and adoption breakdown

<!-- GUI-PERFORMANCE:R21 -->
**R21.** Give each independently verifiable specification requirement a stable ID and
map it to the source clause, all affected consumers, shared interface/owner,
implementation slice, test/scenario, evidence artifact and acceptance rule. Expand
compound requirements where results can differ. Maintain both directions so every
requirement and every consumer has proof and every proposed implementation slice has a
requirement. No missing row, umbrella document citation or prose-only claim may
substitute for individual coverage.

<!-- GUI-PERFORMANCE:R22 -->
**R22.** Publish bounded future slices for measurement completion, window
scheduling/invalidation, retained preparation/uploads, shared dialog/control adoption,
and native/endurance qualification. Distinguish measurement preparation required before
architecture approval from later implementation instrumentation. Define exact scope,
dependency, interfaces, per-slice resource/visual/UX evidence and rollback boundary.
Require baseline comparison at each slice; an unexplained regression blocks advancement.
Reuse existing infrastructure; no framework migration, dependency, render-thread or
visual redesign is authorized here.

<!-- GUI-PERFORMANCE:R23 -->
**R23.** Require tests through actual production handlers for zero unrelated-window
redraws, zero hidden-workspace preparation, no unchanged geometry uploads, no authored
rebuild on warm camera changes, no-op boundary input, safe geometry replacement
identity, cache release, bounded recovery and clipping/hit/focus parity. Map these and
every lifecycle/workload case to the requirement matrix. Simulated cache-reuse tests
alone are insufficient. GPU proof runs serially; the concurrent-test crash remains
separately tracked, and serial success must not be reported as its resolution.

<!-- GUI-PERFORMANCE:R24 -->
**R24.** Before closing the specification, expand the blocked implementation reservation
into the approved requirement-linked completion contract with a separate first owner
execution gate, per-slice acceptance and final independent performance replay. The
reservation retains authorization none until the specification blocker closes; then
transition it to specified/owner_decision, unclaimed and unselected. Only a later
explicit execution grant may select an execution substep and authorize code changes.

The owner's subsequent instruction to implement and verify baseline-selected
bounded Rust corrections is a present, narrower grant: the no-op zoom correction
documented in `docs/reviews/gui-performance/zoom-noop-slice.md` may execute against
its written contract and slice proof under existing camera doctrine. It does not
ratify the proposed shared architecture or activate the wider implementation
reservation. Record its actual results without requiring complete adoption or
endurance first; do not ask for this same bounded grant again.

<!-- GUI-PERFORMANCE:R25 -->
**R25.** For each existing GUI consumer name its adoption slice, shared
interfaces/configuration, legitimate exceptions and parity/regression tests. Define the
same entry checklist for future panes and controls, including resource
ownership/release, invalidation, input/focus and qualification. An exception needs a
bounded semantic reason and evidence; it cannot silently create a second generic
scheduler, renderer or control implementation. Preserve downstream dependencies and
independent engine work.

<!-- REQ:GUI-PERFORMANCE-SPEC:GPS-C05 -->
### Complete specification review

<!-- GUI-PERFORMANCE:R26 -->
**R26.** Require a reviewer distinct from the specification author to review the
complete architecture, measurement design, source evidence and consumer adoption
contract. Record reviewer identity, independence from authorship, relevant
rendering/performance expertise, exact reviewed revision/artifact hashes and a
reproducible review procedure. Self-review remains useful but cannot satisfy this
independent review. No reviewer is assigned by this amendment.

<!-- GUI-PERFORMANCE:R27 -->
**R27.** Record findings with stable IDs, severity, affected requirement, evidence,
correction and reviewer-verified resolution. Critical findings (data loss, corruption or
uncontrolled failure) and high findings (credible required performance/visual/UX
failure, false acceptance or a missing required proof method) must be resolved before
approval. Lower-severity residual risks require explicit owner disposition with impact
and bounded follow-up. Unresolved critical/high findings cannot be waived by relabeling
them as limitations or by silently narrowing required daily-Linux scope.

<!-- GUI-PERFORMANCE:R28 -->
**R28.** Prepare an immutable owner packet containing the baseline and alternatives,
complete specification and proposed numbered mechanism decision,
workload/metric/scale/lifecycle matrices, requirement-to-consumer-to-proof mapping,
implementation breakdown, independent review and residual-risk list. Register artifacts
and their complete evidence routes/parity inventories. Require all mandatory rows
populated, proposed numeric budgets explicit and evidence limitations honest; document
existence alone is not semantic completion. Preserve the protected prototype lane.

<!-- GUI-PERFORMANCE:R29 -->
**R29.** Re-review any changed architecture, budget, workload, scope or proof method
after independent review and before owner approval; bind the final packet to the exact
reviewed bytes. In the future implementation contract require a distinct reviewer to
independently reproduce the required native performance results on the same pinned
candidate/fixtures/environment before acceptance. Review of a report alone cannot
substitute for replay; unavailable qualification remains unavailable.

<!-- REQ:GUI-PERFORMANCE-SPEC:GPS-C06 -->
<!-- OWNER:GUI-PERFORMANCE-SPEC:GPS-C06:GPS-C06 -->
### Owner approval boundary

<!-- GUI-PERFORMANCE:R30 -->
**R30.** Present the complete independently reviewed packet with explicit numeric
budgets, qualification scope, remaining lower-severity risks and the separate execution
boundary. Require baseline evidence, feasible measurement methods for every required
daily-Linux row, complete mappings and zero unresolved critical/high findings before
offering approval. Record exact owner approval or revision input durably before
completing this step or ratifying the numbered decision.

<!-- GUI-PERFORMANCE:R31 -->
**R31.** Reconcile every clause disposition and evidence reference in the approval
transaction. Earlier mechanism/budget clauses may remain pending_owner only while this
owner decision is pending; when approved, resolve them against the exact owner-promoted
numbered decision and reviewed outputs. Do not leave dangling pending-owner rows,
substitute generic phase evidence or treat this planning amendment as ratification.

<!-- GUI-PERFORMANCE:R32 -->
**R32.** Approval is specification-only. It does not select, claim, authorize or certify
the implementation successor, nor qualify runtime behavior. Preserve its separate first
owner execution gate and subsequent production proof, independent replay and acceptance
requirements. Owner response format: `GUI-PERFORMANCE-SPEC: approve specification only`
or `GUI-PERFORMANCE-SPEC: revise — <specific correction>`.

## Downstream integration and task boundaries

The structured Frontier and tracker add the implementation prerequisite to
existing S5A build, cross-probe, Inspector, marking-menu, GUI write-path, and
Project Preferences build work. Their existing blockers remain. Shared-surface
specification may still be refined under its own planning authorization; it
must consume the approved foundation before dependent production expansion.
No additional dependency is imposed on unrelated engine work or the current
bounded Preferences QA correction. Ordering/selection changes do not transfer
the other session's ownership or certify its acceptance.

## Amendment completion and evidence honesty

The immediate owner-authorized amendment ends after committing this brief,
research evidence, paired tracker/Frontier records, generated progress,
classifications, traceability, and the read-only research clarification.
It does not start the first specification step or claim it completed.
Use project-state/projection, governance, traceability, applicable parity,
tracker consistency, and staged-file checks. Report baseline failures separately;
do not renew another lane's claim, change hooks/trust, bless unrelated output,
or manufacture validation success to land this work.

The baseline selector reported an expired Preferences claim and unrelated
WDQ production-coverage failure. Their responsible lanes must reconcile them
before the repository-wide selector can be described as valid. This is not
a blocker on the owner's explicitly authorized read-only investigation.

## Promotion candidate — 2026-09-17

The owner subsequently requested prioritizing promotion of the specification
issue to the next required development step. The candidate coverage policy now
adds precisely two rows: specification classification for the planning item,
with `docs/reviews/gui-performance/specification-clauses.json` accounting for the current individually indexed
requirements and historical cases across all six pending steps; and deferred coverage
classification for the blocked,
unauthorized implementation reservation. Coverage's deferred category is not a
change to its Frontier blocked lifecycle. It provides no production permission.

<!-- GUI-PERFORMANCE-IMPLEMENTATION:RESERVATION -->
The implementation reservation remains blocked on specification approval,
without claim, execution grant, source scope or enrollment. Reclassifying it
as product work, publishing the reviewed implementation completion contract,
and selecting its separate owner execution gate must precede execution. No
existing coverage row, source permission, enrollment, gate implementation,
historical proof or installed trust is changed by these two candidate additions.

This dated reservation describes the wider architecture implementation. The
later owner sequencing direction and R24's bounded corrective scope supersede
its blanket prohibition only for that named correction. No WDQ enrollment or
source-permission promotion is required for it after retirement of that gate.

Installed authority still requires a separately approved promotion of these
exact candidate bytes. Readiness is not asserted by writing the candidate
policy. The owner explicitly directed: "Release the expired claim; leave
Preferences pending". The exact released lease and disposition are archived
in `docs/reviews/gui-performance/promotion-owner-direction.json`. Preferences
returns to ready/open with its existing execution authorization, no assignee
or lease, and its unfinished selected step pending. No product completion or
acceptance is recorded, and it is not the canonical next task.

## Audit correction disposition — 2026-09-17

The owner requested correction of all six readiness findings. The amended
requirements address baseline-before-architecture (R02-R04, R10), complete
measurement acceptance (R13-R17, R19-R20), scale/endurance (R11-R12, R18),
lifecycle recovery (R08-R09), individual traceability/adoption (R21-R25), and
independent review with blocking findings (R26-R31). This is a correction of the
specification-development contract, not evidence that those future deliverables
exist. The previously prepared promotion candidate is superseded by these
amended bytes and must not be activated using its old patch/digest.

## Historical corrective-patch prevention contract

The owner requested this amendment after reviewing pointer, zoom and Preferences
repairs at `7bc8a701`, `5f60ef51`, `c08d1ff7`, `f535b8ae`, `579adb6a` and
`a902ed99`, plus measurement record `4965d758`. These are real corrective
changes, not hypothetical examples. Their full revisions and limitations are
recorded in the owning audit source. This amendment strengthens the required
specification outputs; it does not complete a GPS step, ratify a renderer
mechanism or authorize implementation. "Prevention" means the completed
specification and its implementation acceptance must reject each known faulty
behavior; it is not a guarantee that no future defect can be written.

<!-- GUI-PERFORMANCE:R33 -->
**R33 — Encoding and pass ownership (GPS-C02).** Specify dependency identities
and invalidation for encoded draw commands as well as CPU geometry and uploaded
buffers. For unchanged visible command order, resources, pipelines and bindings,
a warm camera change must reuse eligible encoded world draws while updating
camera uniforms and scissors correctly. Changes to those dependencies must
invalidate reuse. Require counters for encoding/rebuilds, batches, uploads,
passes, resolves and submissions. For every consumer explain why each pass and
resolve exists; reject redundant empty/hidden work and preserve the existing
single-pass dialog result or prove an equally correct, justified alternative.
Do not require one pass for scenes whose semantics genuinely require more.

<!-- GUI-PERFORMANCE:R34 -->
**R34 — Shared control geometry and input contract (GPS-C02).** Define one
configurable owner for compatible scrolling/control mechanisms. Specify delta
units and sign, fractional accumulation, physical/logical conversion, bounds,
page size, thumb geometry, pointer grab/cancellation, keyboard reveal and
clipping. Content, thumb, drawn pixels and hit regions must consume the same
resolved viewport. Legitimate terminal row semantics remain an explicit
configuration/exception. Define retained control tessellation, shaped text and
measured widths with complete dependency keys, bounded residency and release;
scroll position alone must not invalidate unchanged shape/measurement content.
No blanket all-history text cache or frame-rate throttle is an acceptable fix.

<!-- GUI-PERFORMANCE:R35 -->
**R35 — Historical case coverage (GPS-C01/GPS-C04).** The HP cases below are
mandatory minimum cases, not illustrations. GPS-C01 must pin each corrective
commit and its first parent, inspect changed production paths and existing tests,
and reconcile the current code. GPS-C04 must map each case individually to a
normative architecture requirement, shared interface/owner, every applicable
consumer, implementation slice, production test, exact workload, raw artifact
and numerical or deterministic acceptance oracle. The inventory is bidirectional:
every behavior-changing hunk in the six repairs needs an HP mapping or an
explicit reviewed non-performance disposition (for example, source extraction
or derived inventory maintenance). No repaired behavior may disappear in a
commit-level umbrella citation. Every case remains required until exact owner
scope disposition; "already patched" is not a proof exemption.

<!-- GUI-PERFORMANCE:R36 -->
**R36 — Demonstrate defect sensitivity (GPS-C04/GPS-C05).** Specify how each
future test detects its historical failure: replay the first-parent candidate
when its fixture/toolchain permits, or introduce the equivalent defect into an
isolated test candidate. Preserve original sources and raw failing trials;
never mutate the owner's running checkout to demonstrate failure. Compare the
same input and expected output on defective and corrected candidates. Record
which mechanism makes the negative case fail. A test that passes both is not
regression evidence. If historical replay is infeasible, document the exact
incompatibility and require a reviewed equivalent negative control; no silent
pass or self-asserted coverage. These are future authorized proof obligations,
not execution instructions for this planning amendment.

<!-- GUI-PERFORMANCE:R37 -->
**R37 — Complete adoption, not optional helpers (GPS-C04).** Inventory main
window, board/schematic panes, Global and Project Preferences, New Project,
Layers, navigator, Inspector, menus/popovers and terminal consumers, including
currently partial surfaces. For each assign scheduling, layout, preparation,
encoding, upload, clipping/input and lifecycle owners; name old paths to remove
or redirect and semantic exceptions to preserve. Specifically reconcile
`App::request_redraw_if_needed`, the New Project workspace-composition branch,
general screen-space uploads, and dialog-local row/control preparation. Their
existence is observed; their current cost requires measurement. Acceptance must
prove production callers adopted the approved interfaces and have no reachable
unjustified duplicate generic path. Future consumer admission must apply the
same checklist before expansion. Reuse proven fixes rather than replacing them
merely to make the architecture look uniform.

<!-- GUI-PERFORMANCE:R38 -->
**R38 — Historical qualification limits (GPS-C03/GPS-C06).** Keep historical
CPU samples, handler replay, pixel tests, owner feedback and concurrent GPU-test
failure separate. Require simultaneous resource and complete input/display
accounting on matched current workloads. The 193-submission/30-second sample
cannot qualify displayed frame rate or UX; unchanged output caused by discarded
input fails even if CPU/GPU duty falls. Owner confirmation of Global Units
scrolling does not qualify Project Preferences, all other consumers, endurance,
or the renderer architecture. Serial GPU proof does not resolve the recorded
concurrent-test SIGSEGV. The approval packet must contain every HP disposition
and feasible proof method; runtime implementation acceptance later requires
actual positive/negative results, independent replay and native owner UX review.

### Mandatory historical regression cases

Each HP marker is an individually reviewable requirement. "Warm" means the
case's required resources have been prepared, with stable relevant dependencies;
GPS-C03 must define exact warmup, input schedule, scale and counters. Zero-work
oracles below permit required state updates and explicitly named changed
resources, not unrelated preparation. Timing targets never replace these
correctness/work-count oracles. All cases apply through real production entry
points; helper-only tests are supplemental.

| Stable case / corrective commit | Required behavior and discriminating proof |
|---|---|
| <!-- GUI-PERFORMANCE:HP01 --> HP01 · `7bc8a701` | Warm stationary-layout pointer hit queries perform zero additional shell solves. Exercise sidebar, board and terminal hit routes with exact hit parity; the uncached predecessor must exceed the solve count. |
| <!-- GUI-PERFORMANCE:HP02 --> HP02 · `7bc8a701` | Width, height, DPI and dock-height changes resolve correct geometry; alternating shell/terminal queries remain warm within the declared supported working set. Resize churn stays bounded; stale-key and repeated-solve negative cases must fail. |
| <!-- GUI-PERFORMANCE:HP03 --> HP03 · `5f60ef51` | Unchanged compiled menu inventory is parsed once per declared owner lifetime, not per frame; closed menus create no menu preparation. Dynamic action availability still updates and mutable menu copies remain independent. Count parses/preparation through redraw and accessibility paths. |
| <!-- GUI-PERFORMANCE:HP04 --> HP04 · `5f60ef51` | Component alias matching during repeated selection queries does not allocate temporary candidate strings. Preserve all aliases and selection identity; allocation/copy observations must distinguish the allocating predecessor. |
| <!-- GUI-PERFORMANCE:HP05 --> HP05 · `5f60ef51`, `f535b8ae` | Every supported native window invokes the required compositor presentation notification in the correct order before successful presentation. Exercise main, both Preferences and New Project hosts; prove native pacing/idle behavior separately from call-order instrumentation. Missing notification is a failing negative control. |
| <!-- GUI-PERFORMANCE:HP06 --> HP06 · `c08d1ff7` | Warm camera-only board and schematic changes do not rebuild eligible encoded world draws when command order/resources/bindings remain unchanged. Camera uniforms and scissors update; changing any actual encoding dependency rebuilds the affected draw representation. Count encoding in the camera handler-to-frame path. |
| <!-- GUI-PERFORMANCE:HP07 --> HP07 · `c08d1ff7` | Adjacent compatible draw ranges may batch only while preserving exact painter/layer order, primitive type and triangle boundaries. Mixed/interleaved commands and fractional pixel comparisons must detect illegal batching; measure draw/encoding work on the real board workload. |
| <!-- GUI-PERFORMANCE:HP08 --> HP08 · `c08d1ff7` | Unchanged immutable board/schematic vertex sources produce zero repeated world upload bytes. Equal-length replacement, resource recreation and changed geometry must upload correct content; allocator address reuse must never establish false identity. Prove retained ownership plus replacement/eviction lifetime. |
| <!-- GUI-PERFORMANCE:HP09 --> HP09 · `c08d1ff7` | Closed terminal/console consumers cause no screen snapshot, console inspection preparation or hidden-world copy during zoom/hover. Reopening shows accumulated current content without lost terminal state. Account for background terminal processing separately; hiding render work must not stop the terminal core. |
| <!-- GUI-PERFORMANCE:HP10 --> HP10 · `c08d1ff7` | Board wheel routing outside Layers does not prepare the Layers scene or obsolete frame before handling camera input. Wheel over Layers follows its own semantics and cannot zoom the board. Count preparation through both actual event routes. |
| <!-- GUI-PERFORMANCE:HP11 --> HP11 · `f535b8ae` | A changing Preferences wheel event redraws only its owning window; no-op boundary/horizontal-only events request no content frame. Required final input state is retained during coalescing. Exercise both Preferences plus simultaneously open main/other windows and detect the broadcast-redraw predecessor. |
| <!-- GUI-PERFORMANCE:HP12 --> HP12 · `f535b8ae` | Global and Project Preferences preparation consumes dialog data only: zero workspace clones, board polygon/index rebuilds, retained world resolves, hidden terminal preparation or full-shell composition. Preserve settings effects, controls and hit parity across scale/scroll states. |
| <!-- GUI-PERFORMANCE:HP13 --> HP13 · `579adb6a` | An equivalent dialog-only frame uses the justified minimal pass/resolve schedule; the historical four-pass/four-resolve empty-work path must fail structural acceptance even if timings happen to pass. Compare native pixels, antialiasing and painter order against the approved reference. |
| <!-- GUI-PERFORMANCE:HP14 --> HP14 · `579adb6a` | Repeated scroll-away/return of unchanged dialog labels within the declared cache working set causes zero additional shaping after warmup. Count buffer creation/shaping, bound actual CPU/GPU residency and key storage separately, and test eviction, reopen and changing-label churn. A one-frame-only dialog cache must fail. |
| <!-- GUI-PERFORMANCE:HP15 --> HP15 · `579adb6a` | Repeated exact label measurement reuses widths keyed by text, face and size plus any additional supported font/layout dependency. Widths match uncached shaping; changed keys recompute; bounded label churn/eviction releases ownership. The per-call font-shaping predecessor must fail the warm work-count oracle. |
| <!-- GUI-PERFORMANCE:HP16 --> HP16 · `579adb6a` | Convex rounded controls do not invoke the general holed/concave scanline algorithm or repeat invariant arc work. Preserve contour and fractional-position pixels. General concave/holed design geometry retains its correct algorithm. Count control tessellation work and test geometry replacement, not just identical screenshots. |
| <!-- GUI-PERFORMANCE:HP17 --> HP17 · `579adb6a` | Shared text-cache eviction/reordering cannot leave a valid-looking stale workspace glyph signature after dialog rendering. Switch dialog/workspace paths with cache pressure and changed text; stale/missing glyphs must fail. Preserve bounded terminal two-generation churn or an independently qualified equivalent policy. |
| <!-- GUI-PERFORMANCE:HP18 --> HP18 · `a902ed99` | Every finite fractional pixel/line delta contributes to the specified continuous offset until clamped. Repeated sub-threshold input and rapid sign reversals must move correctly; per-event rounding to whole rows must fail. No dead zone or discarded residual may be used to lower CPU. |
| <!-- GUI-PERFORMANCE:HP19 --> HP19 · `a902ed99` | Native physical pixel deltas are not scaled twice; line deltas use the declared logical step converted exactly once. Pointer hit coordinates and viewport geometry agree at 1x, fractional and 2x DPI. Wrong sign or scaling must fail final-offset and hit-target oracles. |
| <!-- GUI-PERFORMANCE:HP20 --> HP20 · `a902ed99` | Scroll maximum is content extent minus visible viewport extent, clamped at zero; thumb fraction/endpoints use the same values. Exercise short content, final row, search groups, expanded choices/explanations and resized windows. No blank overscroll, growing/shrinking thumb from visible-row counting or mismatch at either endpoint. |
| <!-- GUI-PERFORMANCE:HP21 --> HP21 · `a902ed99` | Thumb press/drag preserves grab position, track clicks page correctly, and release/focus loss cancels capture. Resize/content changes during interaction stay bounded. A painted-only scrollbar and click-through into settings must fail production pointer-event tests. |
| <!-- GUI-PERFORMANCE:HP22 --> HP22 · `a902ed99` | Partially visible controls, text and hit regions share the same clip; pinned search/chrome remain unchanged, text does not enter the scrollbar, and hidden controls cannot activate. Test diagonal/rounded geometry, fractional offsets, expanded rows and exact return-to-top pixels. Vertex clamping that distorts edges must fail. |
| <!-- GUI-PERFORMANCE:HP23 --> HP23 · `a902ed99` | Scroll routing is pane-local, keyboard focus/search navigation reveals the correct row, and section/content/DPI/size changes reconcile bounds without stale focus. Preserve reduced-motion behavior and avoid an application-generated idle frame loop. Test both native Preferences hosts independently as well as their shared helper. |
| <!-- GUI-PERFORMANCE:HP24 --> HP24 · `4965d758`, `a902ed99` | Acceptance accounts for the complete scheduled input stream, expected visible states and display opportunities alongside CPU/GPU work. A low-duty candidate that drops deltas, freezes content or reports submissions as delivered frames must fail. Pin identical workloads; do not combine unlike historical percentages into a performance series. |
| <!-- GUI-PERFORMANCE:HP25 --> HP25 · `a902ed99` | Preserve the unresolved concurrent multi-device GPU-test SIGSEGV as a failure with explicit environment/device topology and follow-up. Serial passing tests qualify only their stated scenario. Specify separate supported multiwindow/shared-device lifecycle qualification; neither result may silently stand in for the other. |

### Completion implications of the historical contract

GPS-C01 cannot finish with an unclassified corrective hunk or an unaccounted
consumer. GPS-C02 cannot finish its draft without HP-linked interfaces,
encoding/pass/control contracts and invalidation dependencies. GPS-C03 must
supply feasible measurement methods and input/output oracles for every HP case;
HP01/HP06/HP08/HP11/HP12/HP14/HP15 zero-work claims need production counters,
not CPU percentages alone. GPS-C04 must deliver the complete consumer adoption
and positive/negative test mapping, including retirement of duplicate paths.
GPS-C05 must independently review that mapping and its defect sensitivity;
GPS-C06 must refuse specification approval for missing required cases or proof
methods. These are completion conditions, not a claim that those outputs exist.
The separately authorized implementation must then execute the positive and
negative proofs and independent native replay before runtime acceptance.

The earlier 32-clause promotion candidate is superseded. This revision contains
42 R requirements and 25 individually indexed HP cases (67 clause records),
including the subsequent owner-directed R39–R42 reopening below.
Existing staged owner directions remain historical authorization records; no
old candidate hash, acceptance digest or prior evidence is rewritten to claim
these additions were previously reviewed or approved.

## Owner sequencing correction — 2026-09-17

The owner authorized measurement first, a concrete shared engineering contract
second, then bounded Rust changes chosen by the baseline. Window-local invalidation
is a candidate rather than a preselected optimization. R02-R03 now permit the
measurements required to complete GPS-C01; they no longer withhold that permission.
The exact direction is recorded in
`docs/reviews/gui-performance/sequencing-owner-direction.json`.

Specification readiness requires defined, feasible proof methods and a measured
baseline, not already passing production implementations. All HP01-HP25 remain
mandatory obligations: map each to its affected slice and execute that slice's
relevant positive/negative, functional, visual and resource proof when implemented.
Complete consumer adoption, cross-slice regression replay and long-session/endurance
qualification are final acceptance conditions. An intermediate slice can land
without claiming completion of untouched obligations. Preserve the existing
specification review/approval boundary and implementation acceptance distinction;
do not insert another approval framework or recreate retired WDQ enforcement.

The concrete draft is `specs/GUI_SHARED_ENGINEERING_CONTRACT.md`. Its E01–E10
define shared ownership, invalidation, reuse/lifetime, input/clipping, measurable
oracles and all 25 HP slice allocations. It remains proposed; incomplete broader
baseline/qualification methods and independent review findings remain visible
before specification approval. Executing one bounded correction is neither
GPS-C01 completion nor renderer acceptance.

## Owner-directed global rendering reopening — 2026-09-19

The owner rejects the approximately 34% resize CPU result and reports flickering
and blinking during testing. Rendering-engine work is explicitly reopened at the
shared-foundation level, including surface reconfiguration and all uncovered
scheduling/resource/test gaps. The exact instruction and current observations
are in `docs/reviews/gui-performance/rendering-reopening.json`. GPS-C01R remains
historical evidence of a limited CPU/work correction; it is not resize UX or
resource acceptance. The open QA issue is `dat-gui-vertical-resize-cpu-toj`.
No existing HP case, product/visual authority, successful correction or historical
receipt is discarded. This amendment adds requirements to the existing roadmap;
it does not create another approval framework or claim a runtime fix.

<!-- GUI-PERFORMANCE:R39 -->
**R39 — Shared native-surface lifecycle (GPS-C02).** Finish E10's single shared
surface scheduling/configuration/acquisition/recovery/presentation contract and
explicit host adapters. Cover main Runtime and the shared dialog surface used by
Global Preferences, Project Preferences and New Project, with pane/terminal damage
routed only to the host. Separate desired size/DPI, configured resources and
presented generation; coalesce obsolete resource work without losing input or
final damage. Specify zero extent, resize, DPI, recovery, minimize and teardown.
Duplicating another per-window resize loop or adding an unused helper fails the
shared-by-construction requirement. Proposed mechanisms retain normal ratification.

<!-- GUI-PERFORMANCE:R40 -->
**R40 — Resize cost and absolute resource expectations (GPS-C03).** Attribute
configuration/waits, attachment/MSAA allocation and lifetime, layout/text,
upload/encoding, submission, GPU and compositor costs separately. Compare minimal
host, shell-only and representative/demanding content at a controlled interaction
rate. Define justified per-workload numerical budgets and fixed-overhead limits
before qualification; approximately 34% CPU is rejected, not an accepted budget.
Lower averages or unchanged final pixels cannot excuse blinking, delayed frames,
extra GPU work, allocation growth or unrelated-window work. Preserve visual
quality and frame/input delivery; do not declare success through reduced quality
or freezing rendering. Actual backend evidence and tracing/capture overhead are
required. E07's existing numerical candidates remain unratified.

<!-- GUI-PERFORMANCE:R41 -->
**R41 — Temporal visual and presentation oracle (GPS-C03).** Define and validate
continuous native-resize output measurement for both axes/corners, reversals,
DPI/display transitions and window lifecycle. Require detection of unintended
blank frames, disappearing/stale content, extent mismatch and frame-age/latency
failure during motion. Prove measurement sensitivity with negative controls and
known capture cadence/loss; undersampled or unverified capture is inconclusive.
Settled screenshots, render counters and successful smoke calls are insufficient.
Keep native Wayland and X11/Xwayland qualification distinct; observation during
an artificial smoke does not prove the normal application's flicker cause.

<!-- GUI-PERFORMANCE:R42 -->
**R42 — Global adoption and regression closure (GPS-C04/GPS-C05).** Map R39–R41
and E10 individually to shared owners, every current native host, pane/terminal
consumers, test paths, artifacts and removal of competing legacy code. Correct
smoke reentry and native-size validation before using it as UX proof. Require
work-count and temporal negative controls plus independent native replay; include
surface/attachment churn, cross-window invalidation, text/upload reuse, recovery
and final damage. A local optimization cannot close global adoption. Keep the
resize QA issue unresolved until both temporal UX and resource criteria pass;
complete adoption and endurance remain final acceptance, with relevant proof
executed for each implementation slice.

GPS-C02 must draft this shared lifecycle and reconcile the current main/dialog
paths. GPS-C03 must define the missing absolute resource and temporal methods;
GPS-C04 must make adoption and retirement executable for every consumer. The
existing measurement-first, contract-before-code, bounded-implementation sequence
remains. This amendment itself neither implements the contract nor completes
GPS-C02–GPS-C06 or wider engine acceptance.

## Subsequent owner disposition: limited initial qualification

Exact responses: `docs/reviews/gui-performance/initial-qualification-owner-direction.json`.
The owner approved initial small-project qualification with explicit limits and
excluded flicker/displayed-frame latency acceptance. Apply the R12/R14 scope
disposition in `GUI_PERFORMANCE_ACCEPTANCE_MATRIX.md`: pinned T0/T1 workloads
only, T2 unaccepted; temporal output, frame-age/pacing and displayed-latency
qualification deferred without a passing result. No capture-calibration task is
selected by this direction. GPU accounting and other correctness/resource proof
remain required; the answers are not a broader measurement waiver.

R39–R42 and HP01–HP25 remain engineering and final-acceptance obligations. GPS-C03
may prepare the limited initial acceptance contract without completed demanding
project or temporal calibration, but must resolve its remaining in-scope methods,
counts and numerical rules. GPS-C04 must map deferred cases explicitly; GPS-C05
and GPS-C06 must review and disclose the qualification exclusions. No initial
result closes resize QA or establishes complete rendering-engine performance.
Mechanism/numeric ratification and implementation authorization remain separate.
