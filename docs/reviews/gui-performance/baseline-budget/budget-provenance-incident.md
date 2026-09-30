# Incident: unsupported GUI timing targets became development gates

Status: factual incident record and owner-requested corrective-action handoff.
This record does not itself ratify a mechanism, alter an acceptance threshold,
authorize execution, or establish hardware infeasibility. Integrate it with the
in-progress PM051 correction and `dat-gui-baseline-budgets-j9w8`.

## What happened

The GPU execution targets of 4 ms p95 and 8 ms p99 entered the performance
specification without a recorded empirical derivation establishing their
feasibility on the reference host. They subsequently became binding acceptance
criteria and, through the later S4 reopening, development-blocking requirements.
Repeated implementation and measurement work pursued those figures before their
basis was adequately challenged.

The owner reports that the figures were initially guidelines, supplied because
the specification needed numbers, and that agents worked around the clock for
approximately ten days pursuing them. This duration and characterization are
owner-reported impact, not an independently measured compute or labor total.
The retained repository evidence establishes the missing documented GPU-budget
derivation; it does not establish the original author's private reasoning.

## Evidence and authority trail

1. Commit `5bba3176` (2026-09-19), introducing
   [the acceptance matrix](../../../../specs/GUI_PERFORMANCE_ACCEPTANCE_MATRIX.md),
   already contains the T1 4/8 ms figures. Its commit proof explicitly records
   that no new runtime measurement or Rust build was performed. The matrix
   explains the CPU target relative to rejected resize usage, but the reviewed
   provenance contains no equivalent measurement or calculation deriving the
   GPU 4/8 ms values. Absence of a derivation in these records is not a claim
   that no measurement of any kind existed elsewhere.
2. [PM045](../../../decisions/PRODUCT_MECHANICS_045_SHARED_GUI_PERFORMANCE.md),
   GP-045-01, originally distinguishes maximum acceptable costs from measured
   achievement. Its alternatives/tradeoffs acknowledge that driver costs might
   make the proposed ceilings unachievable on the reference setup.
3. [RISK-01](../residual-risks.json) preserves that feasibility uncertainty and
   directs failed qualification to evidence and owner review. It did not supply
   an early feasibility gate, effort limit, or stopping decision sufficient to
   prevent prolonged pursuit of an unsupported numerical requirement.
4. [The final owner direction](../final-owner-direction.json) approved the
   reviewed specification and budgets. That established specification authority,
   not empirical feasibility. Later agents treated this authority as sufficient
   grounds to continue enforcing the targets.
5. Commit `997b618e` and
   [the S4 owner direction](../gpu-redraw-s4-owner-direction.json) made the GPU
   correction an S4 exit prerequisite while retaining the numerical gates.
6. The [corrected painter result](../gpu-redraw-proposal/painter-hover-timed-result/report.md)
   reported p95 4.330583 ms and p99 5.701750 ms. The
   [R4 assessment](../gpu-redraw-proposal/r4-native-result/assessment.md) reported
   p95 about 6.700 ms and p99 about 10.205 ms. These remain valid observations at
   their recorded scopes, not matched comparative estimates or proof that the
   original thresholds were justified.
7. [Corrected A1 attribution](../gpu-redraw-proposal/attribution-a1/corrected-result/report.md)
   completed the bounded study but did not demonstrate a universal hardware
   floor or dependable recoverable end-to-end savings warranting another
   renderer redesign. Its uncertainty must not be rewritten as proof that 4 ms
   is impossible, or as permission for indefinite further investigation.

## Failure mechanism and impact

An aspirational numerical target became a binding gate without the evidence
needed to justify enforcement. Reviews checked compliance, measurement rigor,
source ownership and bounded individual experiments, but did not adequately
challenge the target's provenance or accumulated development cost. Repeated
approval and documentation increased the target's apparent credibility without
adding evidence for its attainability.

Real architectural and correctness defects were also found and corrected.
Those improvements retain their value, but do not retroactively justify the
budget or establish that all work was necessary. Diagnostic-plan, launcher,
resource-integration and workload-initialization mistakes added delay. Repeated
stop/approval cycles around routine setup recovery further displaced the actual
engineering question.

The owner reports delayed functionality development, sustained agent effort,
loss of confidence and difficulty understanding the path to completion. Exact
monetary cost, compute consumption and avoidable fraction are not established.
The originating audit/review session also contributed: it repeatedly recommended
locally bounded proposals while acknowledging unresolved host feasibility,
without adequately enforcing an overall value and stopping decision. This was
not solely the implementation session's failure.

This incident is unsupported requirements becoming authoritative, not a finding
that runtime measurements were fabricated. Historical failures, invalid trials,
successful proofs and their limitations must remain intact.

## Corrective direction and prevention

The [owner's replacement direction](owner-direction.json) is authoritative input
to the concurrent specification correction. Its required outcomes are:

- Withdraw unsupported GPU 4/8 ms thresholds as blocking criteria through the
  controlling decision/contracts/roadmap transaction. Preserve correctness and
  historical measured results; do not silently waive adjacent requirements.
- Establish a measured reference baseline from a stable pinned candidate,
  representative workloads, verified delivered input/output, complete timing
  boundaries and instrumentation-effect evidence. Reuse sufficient existing
  evidence and report distributions, counts, variability and limitations.
- Separate observed baseline from acceptable product performance. Replacement
  budgets need a documented rationale using measured behavior, responsiveness,
  supported configurations and explicit headroom or improvement assumptions.
- Require derivation, supporting evidence, applicable configurations and review
  conditions before a numerical threshold becomes blocking. Owner approval
  does not substitute for missing technical evidence. If evidence cannot support
  a number, record it as undecided rather than inventing one to finish a spec.
- Audit adjacent CPU, GPU-duty and resource thresholds for provenance. Audit is
  not authorization to alter them or begin another optimization campaign.
- Bound investigation effort and require an overall product/value decision.
  Repeated failure must reopen review of the requirement as well as the code.
  Do not interpret a stopped setup attempt as a hardware finding or let routine
  recovery permissions replace accountability for diagnostic preparation.
- Reconcile controlling documents and active-agent handoffs so functionality
  work can proceed while explicitly scheduled baseline/budget work remains.

## Reconciliation handoff

The baseline-budget correction session owns PM051, its issue/Frontier changes,
generated PROGRESS, manifest classifications and evidence-route reconciliation.
This incident is classified as historical in the specification governance
manifest, linked from PROGRESS and the controlling prevention rule in CLAUDE.md.
The correction session must review and link it from the numerical correction's
decision or handoff, and include it in that correction's evidence reconciliation
as applicable. This record does not claim that PM051 integration is complete.

At drafting, `project_status.py next` failed because the newly created scheduled
issue `dat-gui-baseline-budgets-j9w8` was not yet in the Frontier. The other
session's dirty tracker and baseline-budget files were left untouched. No
production edits, builds, runtime tests or new performance claims accompany this
record. Incident closure requires verified reconciliation and delivery of the
corrected authority to active agents, not just creation of this document.
