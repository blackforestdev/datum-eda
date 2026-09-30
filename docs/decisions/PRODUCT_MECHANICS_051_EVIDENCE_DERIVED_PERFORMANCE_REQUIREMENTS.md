# Product Mechanics 051: Evidence-derived performance requirements

Status: ratified owner amendment. Exact direction:
`docs/reviews/gui-performance/baseline-budget/owner-direction.json`.
Tracking: `dat-gui-baseline-budgets-j9w8`, GUI-PERFORMANCE-BASELINE;
related implementation/qualification: `dat-gui-performance-implementation-vkq`.

## Decision and precedence

**GP-051-01 — Withdraw unsupported GPU frame gates.** The GPU execution p95
4 ms and p99 8 ms figures were not supported by a validated derivation. They are
withdrawn as blocking acceptance criteria, including their use in the MET-03
workload table, recovery references, and GPI-S4/S5 exit conditions. Wherever the
GPU 8 ms p99 figure appears with another p95 target, that 8 ms component is withdrawn;
no other value changes by implication. Exceeding these figures alone does not
establish a product defect. Historical measurements, configurations, correctness
results and contemporaneous dispositions remain recorded at their original scope;
old reports are not rewritten into passes or failures under a new criterion.

This amendment supersedes conflicting numerical/sequence language in PM045,
PM050, the performance recovery/engineering/acceptance/implementation contracts,
and prior proposal/approval/measurement packets. It does not invalidate their
correctness obligations or erase demonstrated renderer/diagnostic defects.
CPU event/preparation p95 4 ms/p99 8 ms is a different adjacent requirement: it is
audited, not silently withdrawn with the GPU figures. Existing GPU-duty, memory,
cache/admission, other latency and work-count requirements are unchanged unless
explicitly dispositioned later. Zero lost input, exact 8× output, painter order,
invalidation/recovery, lifetime correctness and authoritative state remain.

**GP-051-02 — Measure a reference baseline; do not manufacture acceptance.**
Establish a stable, source/binary/fixture/environment-pinned reference candidate
on the reference host. Select by implementation/stability suitability before
new timings, not by the fastest historical number. Use representative admitted
workloads, verified input and output, complete timing boundaries including
uploads/submissions/final copies, and measured instrumentation effects. Reuse
adequate evidence; a handoff is not grounds for repeating it. Record why each
missing observation is needed and its finite collection/stop limits before entry.
Preserve every attempted trial and any invalidity reason; no favorable-run subset,
adaptive optimization sweep, universal overhead subtraction or pooled trial
summary hiding variability. Report each trial's nearest-rank p95/p99, N, max,
configuration, validity, variability and limitations. Unsupported metrics stay
unmeasured or inconclusive, not zero.

**GP-051-03 — Baseline and budget have different authority.** The baseline is an
observation of performance. It does not automatically establish acceptable
performance, a minimum achievable cost or a target with an arbitrary multiplier.
A proposed budget must connect the measured distributions and their variability
to demonstrated responsiveness, supported hardware/configuration expectations,
and an explicit rationale for headroom or required improvement. GPU execution
is not input-to-observed-display responsiveness. Missing temporal observation or
hardware coverage limits the possible claim; do not expand initial qualification
by assertion. If the evidence cannot justify a number, the budget is explicitly
**undecided**. An undecided outcome completes an honest assessment, not a numeric
acceptance rule or runtime qualification.

**GP-051-04 — Owner ratifies blocking status.** The owner must approve the exact
budget rationale, evidence scope and review conditions before a replacement
threshold becomes blocking. Independent technical review does not substitute
for owner ratification. Recording a measured baseline, completing a plan or
finishing the specification does not imply approval. At this amendment no
replacement GPU p95/p99 figure is proposed or binding.

**GP-051-05 — Precedent for every future numerical performance requirement.**
Every new or substantively revised binding performance threshold must cite:
its derivation; supporting evidence with method validity and variability;
applicable workload/tier/hardware/backend/scale/consumer configurations; and
review conditions, including invalidation triggers and approval authority.
State whether it is a responsiveness requirement, resource/admission guard,
regression trigger or measurement-protocol bound; those are not interchangeable.
Do not present arithmetic, convenience, historical convention or an observed
maximum alone as feasibility or product rationale. Reference-host policy must
not silently become all-hardware policy. Changes to workload, quality, candidate,
backend/driver, timing boundaries, instrumentation or supported hardware trigger
review of affected evidence and rationale. Reuse unaffected evidence.

Legacy CPU/GPU-duty/resource thresholds receive a provenance audit. Missing
support is recorded as a gap, not an automatic waiver, silent cap change or
fabricated derivation. Safety/correctness and bounded-lifetime semantics remain
independent of a chosen numerical cap. Existing thresholds other than the
explicit GPU withdrawal retain their current authority until owner disposition.

**GP-051-06 — Functionality is no longer globally gated on this work.** Remove
`dat-gui-performance-implementation-vkq` as a blanket hard dependency of UVT S5A,
cross-probe, full inspector, marking menu, GUI write path and Project Preferences.
Preserve those tasks' other dependencies, planning/execution/owner boundaries,
nonmutation/mutation authority, local regression and all correctness obligations.
An actual correctness or safety defect that affects a feature is an explicit
feature prerequisite; neither this amendment nor a nonblocking baseline excuses
it. Do not close the performance issue or mark S4/S5 complete as a shortcut.

GUI-PERFORMANCE-BASELINE is an explicitly scheduled, nonblocking planning lane.
Residual implementation/correctness/qualification remains on
GUI-PERFORMANCE-IMPLEMENTATION, also nonblocking globally, with a bounded
reconciliation step before new work. The canonical functionality task becomes
UVT-S5A-BUILD / S5A-C01 under its existing planning authority. Its implementation
still needs its existing owner execution disposition. No new functionality or
renderer implementation is authorized by this scheduling amendment alone.
Parallel lane marking does not authorize concurrent agents or bypass claims.

## Bounded delivery and remaining authority

`specs/GUI_PERFORMANCE_BASELINE_BUDGET_PLAN.md` defines the evidence inventory,
missing-only protocol, observed baseline, budget rationale and owner disposition
steps. The current amendment performs governance, evidence reuse and initial
audit; it does not assert that a representative validated baseline is already
complete. No new runtime campaign occurs in this governance transaction.
The owner's direction authorizes bounded baseline work; routine setup repair
within its declared collection scope does not need repeated permission. Freeze
the missing-only protocol before execution and obtain owner approval of any new
scope outside it. Replacement binding budgets always need the explicit decision.

Full DRM duty/lifetime, wider correctness/endurance and supported-configuration
qualification remain separately visible. CPU event-loop optimization and renderer
redesign are not baseline collection. No dependency/license or protected visual
prototype change is introduced.

## Prevention and incident reconciliation

The controlling prevention rule in `CLAUDE.md` (numerical performance requirements
must have an evidentiary basis) and the preserved
[budget provenance incident](../reviews/gui-performance/baseline-budget/budget-provenance-incident.md)
are reconciled by this decision, the explicit baseline lane and the active-agent
handoff. The incident records owner-reported impact separately from independently
measured facts. Historical evidence is not rewritten; this governance correction
does not itself close the remaining baseline, budget or runtime qualification work.
