# CPU requirements: derivation before investigation

Current scheduling disposition: this completed assessment is retained at its
recorded scope. The subsequent owner reset and
[resize investigation](resize-investigation.md) supersede its proposed
pointer/control packet and numerical-basis prerequisite for descriptive
investigation. Budgets remain undecided; no repeat audit, derivation or pointer
trial is a resize prerequisite. See `owner-direction.json` → `resize_reset`.


<!-- EVIDENCE:GUI-PERFORMANCE-IMPLEMENTATION:CPU-DERIVATION-ASSESSED -->
The owner requires derivations before CPU investigation. This supersedes the
prior audit's recommendation to prepare a six-launch ordinary-mode packet next.
That packet is postponed, not authorized. No measurement, profiling, optimization,
production change or new acceptance threshold follows from this assessment.

## What can actually be derived

Sources: PM045 rationale (paragraph beginning “The 10% T1 active CPU proposal”),
GUI_PERFORMANCE_ACCEPTANCE_MATRIX MET-02/03/05/06 and MEM-02/STAT-01, PM051
GP-051-03–05, and `../baseline-budget/threshold-audit.md`. Historical and current
CPU evidence is pinned in `evidence.json` and interpreted in `audit.md`.

Let D be CPU duty as a percentage of one core and r the actual acknowledged
semantic actions per second. A duty allowance implies average CPU per action
C = 10D/r milliseconds, only when the accounted work and action denominator
match. This is a units conversion, not a derivation of acceptable duty, a tail
limit, or a statement that all CPU belongs to one action handler.

| Existing figure | Recoverable derivation | Missing justification |
|---|---|---|
| T1 duty10% | PM045 intentionally chose a value below owner-rejected32–35% resize CPU | Why10 rather than another value; transfer from resize to pointer/pan/zoom; responsiveness, host capability and concurrent-work allowance |
| Idle/clamp1%, T0/Preferences5%, mixed15% | Listed policy choices; mixed15 can be described arithmetically as10+5 but the record does not establish that as its product derivation | Allowed background work and idle cost, per-consumer requirements, PTY workload/host feasibility; do not fabricate provenance from convenient arithmetic |
| 1.667 ms at60/s;0.833 ms at120/s for10% | 100/60 and100/120 ms, conditional on the parent10% and actual semantic rate | Parent budget remains unsupported; requested pointer positions are not necessarily acknowledged actions |
| 0.833 ms at60/s for5%;0.417 ms at120/s for5% | 50/60 and50/120 ms | Same unvalidated parent5% budget |
| Clamp0.167 ms at60/s for1% | 10/60 ms per scheduled no-op input | No-op is not a semantic state change. MET/STAT require elapsed/scheduled-input denominator; not invented acknowledged actions; parent1% unvalidated |
| Pane50 ms at2 actions/s for10% | 100/2 ms | Parent10% and actual recipe/action-rate applicability; not a general50ms responsiveness derivation |
| Controls5 ms; warm open50 ms/close20 ms CPU | No independent derivation located; historical scoped measurements contain both failures and later passing window trials | Required user experience, supported configuration, end-to-end critical path and justified CPU share |
| CPU p95 4 ms/p99 8 ms; stall50 ms | No validated derivation located. A60Hz refresh period is16.667ms, but assigning fractions of it to CPU does not justify these percentiles | CPU/GPU/compositor critical path, scheduling overlap, latency objective, tail coverage and empirical feasibility. CPU elapsed latency and summed process CPU are different quantities |
| Recovery100 ms CPU | Existing limit plus diagnostic recovery evidence | User-visible recovery objective, preserved work, backend/device costs and independently validated CPU accounting |
| Repeatable5% regression;3absolute trials/7pairs | Existing review/protocol choices with defined STAT-01 calculation | These counts do not establish statistical power or product tolerance and do not derive any acceptance ceiling |

**Conclusion:** some per-action arithmetic is valid conditional on parent duty,
but no reviewed evidence validates the parent duty or CPU tail/transition budgets
as product acceptance numbers. Current instrumented observations cannot repair
that gap. None is promoted to an evidence-derived CPU requirement here.

## What a defensible derivation must contain

Each proposed binding requirement must have a single record containing:

1. The product need it protects: responsiveness, resource sharing, idle behavior,
   recovery usability or regression policy. Specify the user task and expected
   outcome; “lower than an old bad measurement” is insufficient.
2. The supported configuration: hardware class, backend/driver, workload, input
   semantics, quality, concurrent work and scope. A reference-host observation
   is not a supported-hardware policy.
3. The model connecting that need to this metric. Duty is a resource allowance;
   CPU/action needs actual acknowledged rates; event elapsed tails and total CPU
   have distinct boundaries. Do not add independently computed percentiles or
   allocate an arbitrary fraction of a refresh interval.
4. Adequate existing feasibility evidence with source/binary/method pins,
   instrumentation effects, trial distributions, counts and variability. State
   what cannot be inferred. A measured maximum plus arbitrary headroom is not
   acceptable. Historical corrections show feasibility only at their scope.
5. Justification for headroom or required improvement, review/invalidation
   conditions and explicit owner ratification under PM051. If evidence is missing,
   label the proposed number undecided; do not backfill a convenient value.

The present record lacks validated CPU-specific user-response/resource-sharing
objectives, supported-hardware expectations and clean representative feasibility
for these exact limits. The accepted pointer subset does not supply them.
Those missing policy/evidence premises must be resolved before the proposed CPU
investigation packet. If existing evidence proves insufficient, any later
feasibility collection needs its own explicit owner scope; it cannot be inferred
from this derivation-first instruction. This is not permission to start another
baseline or optimization campaign to make a guessed number appear achievable.

## Authority and scheduling disposition

GPI-CPU-DERIVATION records this completed assessment, **not completed validation**.
GPI-CPU-DERIVATION-DISPOSITION precedes GPI-CPU-SCOPE and GPI-CPU-ESTABLISH.
No CPU harness work, measurements or optimization advance while the derivation
boundary remains unresolved. Original CPU issue `dat-gui-pointer-layout-x1r`
remains open. S4 remains complete; S5 qualification and method disposition remain
separate. GBB-P05 retains baseline/budget rationale and owner ratification; this
assessment is reusable evidence for that lane, not a duplicate collection effort.

PM051 explicitly preserves existing CPU criteria until owner disposition. This
assessment does not silently withdraw them. It also does not use their unsupported
derivations as a basis for initiating optimization. Replacement requirements stay
undecided, and historical measurements retain their recorded scope.

<!-- OWNER:GUI-PERFORMANCE-IMPLEMENTATION:GPI-CPU-DERIVATION-DISPOSITION:BASIS -->
The outstanding owner boundary is the CPU requirement basis, not permission to
run the six-trial packet. Supply or ratify the product/resource-sharing objectives
and supported-hardware scope from which an evidence-backed budget can be derived,
or keep replacement CPU budgets undecided and CPU investigation deferred. Do not
approve a guessed numerical threshold. Missing feasibility evidence must remain
explicit and needs a separately scoped decision if collection becomes necessary.
