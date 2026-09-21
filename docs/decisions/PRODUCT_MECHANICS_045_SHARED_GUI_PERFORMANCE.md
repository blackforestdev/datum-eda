# Product Mechanics 045: Shared GUI performance and resource ownership

Status: **ratified specification doctrine** by the owner at GPS-C06.
Issue: `dat-gui-performance-spec-u9n`. Implementation reservation:
`dat-gui-performance-implementation-vkq` remains separately gated.

## Problem and evidence

Pointer, camera and Preferences fixes exposed repeated preparation, uploads,
dialog composition and inconsistent ownership. Native resize QA then exposed
approximately 32–35% of one CPU core and content blinking/jumping. Historical
partial reductions and settled screenshots did not qualify temporal output or
resolve that defect. The C01 evidence, six corrective commits and 25 historical
regression obligations remain in the performance evidence route; none is erased
or retroactively accepted by this proposal.

The existing retained renderer, shared viewport toolkit and dialog-only path
provide working infrastructure. A Rust base-class hierarchy or replacement
framework would not itself fix invalidation or resource lifetimes. The decision
therefore proposes explicit shared owners and consumer adapters, with measured
cost and correctness deciding each bounded implementation change.

## Proposed decision

**GP-045-01 — Performance and fidelity together.** Apply the proposed numerical
CPU, GPU and memory ceilings and exact work-count/correctness predicates in
`specs/GUI_PERFORMANCE_ACCEPTANCE_MATRIX.md`. Low utilization with lost input,
stale content, reduced AA or omitted work fails. A budget pass does not justify
known avoidable work. These are maximum acceptable costs, not a claim of a
provable minimum or a measured achievement.

**GP-045-02 — Shared host ownership.** Adopt E01–E12 in
`specs/GUI_SHARED_ENGINEERING_CONTRACT.md`: one application coordinator for
native host scheduling and device/queue coordination; per-host damage and
surface generations; shared bounded recovery. Main, Global Preferences, Project
Preferences and New Project use the same lifecycle implementation. Panes report
damage to their host and retain distinct cameras, input and document authority.
No alternate generic scheduler or configuration loop remains behind an adapter.

**GP-045-03 — Dependency reuse.** Retain immutable world ownership, correctly
keyed encoded draws, shaped text and controls; invalidate only actual changed
dependencies. Count allocation capacities, uploads, cache keys/payloads and
in-flight generations. Bound eviction and release without evicting authoritative
design or terminal state. Preserve painter order, typography and visual doctrine.

**GP-045-04 — Shared interaction geometry.** Continuous controls use the common
scroll/clip/hit/focus contract. Terminal and Layers keep meaningful discrete rows.
Paint and input derive from coherent geometry; partial/hidden content cannot
receive inconsistent input. TerminalCore, PTY, project settings and the canonical
engine mutation path retain their existing authority.

**GP-045-05 — Proof and adoption.** Use
`specs/GUI_PERFORMANCE_IMPLEMENTATION_CONTRACT.md` and its individual map for
all 14 current consumers. Tests exercise production adapters and deliberately
faulty controls. Complete adoption requires retiring competing paths. Relevant
proof accompanies each bounded slice; final independent native replay and
endurance follow adoption. Instrumentation conformance is required before relying on the corresponding
measurement. S0 establishes the matched baseline, verified input/final state,
and attributable work needed to select and verify the first bounded S1 change.
Complete ADM/GPU/ACC accounting is developed and verified with its production
owners in S1–S4; affected correctness, resource-lifetime, zero-work and cache-bound
proof accompanies each change. Full admission and accounting remain prerequisites
to runtime tier/budget qualification and S5 completion. Missing results remain
unqualified; neither S0 closure nor a component migration declares them satisfied.

**GP-045-06 — Explicit initial scope.** The owner has limited initial
qualification to pinned F-DOA/T0/T1 configurations and excluded flicker,
displayed-frame latency/age/pacing and larger-project qualification. Unadmitted
resolved schematic content remains unqualified. GPU accounting, memory, static
appearance, input/focus, clipping/hits and final state remain required. These
limits permit an honest initial specification; they do not establish complete
visual responsiveness or enterprise capacity. The later owner amendment removes E10 temporal calibration as a development
prerequisite. Resize QA remains open until its resource and temporal criteria
pass; the kernel/driver/hardware explanation remains unconfirmed.

**GP-045-07 — Separate execution.** Ratification approves only the exact reviewed
specification, proposed mechanisms, budgets and adoption plan. It does not
authorize S0–S5 execution, accept an old dirty Rust candidate, add a dependency,
alter a protected visual prototype, change backend policy or close product QA.
The prepared implementation completion contract starts with the existing owner
execution gate. Its installation when the specification blocker closes is an
explicit roadmap transaction, unclaimed and unselected. GPS-C01R is retained on its existing open bug as nonblocking investigation,
outside the completed specification item.

## Alternatives and tradeoffs

| Alternative | Evidence-based disposition |
|---|---|
| Replace renderer/toolkit or introduce a render thread now | Rejected for this scope: current evidence identifies ownership/reuse/lifecycle gaps, not a demonstrated need for replacement; adds migration, synchronization and licensing cost without a measured causal benefit |
| Keep fixing each pane independently | Rejected: observed main/dialog asymmetry and recurring cases require shared ownership and all-consumer adoption |
| Add larger caches or throttle input until CPU falls | Rejected: unbounded residency and dropped output can counterfeit resource gains; measure retained bytes and preserve input/output |
| Shared incremental ownership and retained reuse | Proposed: existing services and production handlers are adoption points; bounded slice proof and rollback expose regressions early |
| Call the present limited measurements full acceptance | Rejected: missing temporal, demanding-tier and runtime accounting evidence remains explicit |

The 10% T1 active CPU proposal is substantially below the owner-rejected resize
cost; T0 and Preferences target 5%, idle 1%. GPU and per-action budgets prevent
moving CPU work invisibly to another process or device. RSS and cache caps use
the observed small-board baseline plus explicit headroom; they are admission
ceilings, not reservations. Driver submission costs may make a proposed ceiling
unachievable on the reference setup. That is a failed qualification requiring
evidence and owner review, not permission to quietly raise the ceiling, drop
events or reduce quality. No larger-tier numbers are ratified without admission.

## Review and approval

The immutable owner packet binds this proposal, all contract/matrix bytes,
baseline/alternatives, individual adoption/proof mapping and independent review.
Any substantive change after review requires delta review and a new packet hash.
GPS-C06 requires the owner's exact specification-only disposition. Until that
record exists this decision remains proposed and has no execution authority.
Dependency/license impact: none; Product Mechanics 029 remains controlling.
Product Mechanics 023, the Rendering Book, terminal doctrine, Preferences authority
and the protected prototype lane remain controlling in their existing domains.

## GPS-C06 ratification record

<!-- EVIDENCE:GUI-PERFORMANCE-SPEC:GPS-C06-RATIFIED -->

The owner replied exactly `GUI-PERFORMANCE-SPEC: approve specification only`.
`docs/reviews/gui-performance/final-owner-direction.json` binds that approval to
the reviewed packet and resolves all 67 clause and six residual-risk dispositions.
The proposed mechanisms, numerical budgets and adoption/proof definitions above
are now approved specification requirements. Pre-ratification wording retained
in this decision and the frozen packet describes the reviewed proposal; this
record supplies its current approval state. Runtime results remain unaccepted.
No implementation grant or resize closure follows from this ratification.

## Owner amendment: resize investigation is nonblocking

The subsequent owner instruction in
`docs/reviews/gui-performance/resize-deferral-owner-direction.json` pins
`dat-gui-vertical-resize-cpu-toj` as unresolved investigation and removes resize
resolution, kernel/driver attribution and completed temporal calibration as
development gates for S0–S5. This amendment supersedes earlier E10 pre-change
calibration requirements and GPS-C01R blocking sequence. It does not mark resize
budgets, flicker or displayed-frame criteria passed or change their numerical
limits. The Linux-kernel/Intel-driver/hardware explanation is an unconfirmed
owner hypothesis, not an established root cause.

Shared application lifecycle and reuse fixes may proceed under the separate
execution grant, with bounded relevant production regressions, available
before/after resource observations and final-state/extent/hit/focus/PTY/static
correctness checks. Missing temporal proof is reported as unqualified. It cannot
be used to claim resize closure, but it does not stop other development.
All non-resize performance/correctness, adoption, independent replay and
endurance requirements remain. S5 can conclude only its explicitly limited
non-resize qualification; full resize resource/temporal closure stays on the
pinned issue. No new kernel investigation or renderer execution is authorized
by this amendment.
