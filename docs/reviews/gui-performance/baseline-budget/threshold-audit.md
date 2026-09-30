# Initial adjacent numerical-threshold provenance audit

Scope: MET/MEM/GPU/STAT and affected PM045/PM050 limits. This is a source/evidence
provenance audit, not new runtime qualification or a CPU optimization investigation.
PM051 withdraws only the GPU 4 ms p95/8 ms p99 figures. Every other threshold below
keeps its current status and scope until explicit owner disposition.

Evidence reviewed: PM045 rationale; acceptance matrix MET-01–06, MEM-01–03,
GPU-01–03, ACC-01–03, STAT-01; shared engineering E01–E12; implementation contract;
PM050's r4 image arithmetic; preserved R4/A1 and earlier small-board observations.
The unchanged full performance evidence-route review is reused, with this amendment
reconciling historical numerical dispositions rather than rewriting evidence.

| Family / source | Existing number(s) | Derivation/evidence status | Needed disposition/evidence; current authority |
|---|---|---|---|
| GPU frame tails, MET-03/recovery/S4 | 4 ms p95; 8 ms p99 | No validated product/host derivation established; owner explicitly withdraws | Withdrawn as blockers. R4 measurements remain valid observations. Replacement undecided |
| CPU duty, MET-03/PM045 | idle 1%; T0/Preferences 5%; T1 10%; mixed 15% | T1 justified qualitatively as below rejected32–35%resize; not a demonstrated responsiveness/hardware feasibility derivation. Other choices lack complete local derivations | Need representative CPU totals, input rates, responsiveness and hardware rationale; unchanged |
| CPU/action, MET-03 | .167/.417/.833/1.667 ms; pane 50 ms; open 50 ms/close 20 ms; other controls 5 ms | Some values are duty/rate arithmetic (e.g 10%/60s⁻¹=1.667 ms); valid conversion does not justify parent duty. Transition ceilings lack equivalent shown derivation | Preserve denominators and full GUI+engine accounting; validate action costs and product rationale; unchanged |
| CPU event/preparation/encoding, MET-05/E10/R19 | p95 4 ms, p99 8 ms; stall 50 ms | Separate CPU thresholds, not GPU gates. No validated derivation established by A1; fixed preparation bypass is not event-loop evidence | Audit further without opening CPU optimization; unchanged |
| Display responsiveness, MET-05 | 33.4/50 ms tails; 100/150 ms transition tails; 95%within 20 ms at60Hz; final state 50 ms | Refresh-related rationale is suggestive, not validated displayed-response proof. Initial temporal qualification explicitly excluded | Need trustworthy displayed-output evidence and product expectation; existing exclusions and future requirement status unchanged |
| GPU engine duty, MET-03 | idle/no-op zero active delta; 5/10/25/30%active ceilings | Zero-work predicates have structural rationale. Nonzero ceiling derivations/complete reference feasibility not established; full DRM lifetime accounting still missing | Preserve zero-work/correctness and numerical status; quantify duty only with complete clients/lifetimes. No automatic relaxation |
| Other GPU tails, MET-03 | T0 .8/1.6 ms; Preferences1.6/3.2 ms; mixed 5 ms p95 | No complete workload/host derivation demonstrated here | Audit rationale/feasibility; unchanged. Mixed 8 ms p99 withdrawn explicitly |
| GUI RSS, MEM-02 | 256 MiB steady/384 MiB peak | Approximately 156 MiB historical pan RSS supports a baseline+headroom proposal; not representative admission/peak/concurrent-host qualification | Cite source/candidate and justify headroom against all supported states; unchanged |
| Additional hosts, MEM-02 | 64 MiB incremental; 512 MiB combined; return within 16 MiB/5s | Policy bounds; complete concurrent-host/driver/allocator derivation not established | Need live/retiring/OS footprint and close/recovery distributions; unchanged |
| Engine RSS, MEM-02 | 256/384 MiB | Frozen T1 model scope explicit; budget derivation not shown by GPU evidence | Complete process-lifetime accounting and representative model evidence; unchanged |
| CPU world history, MEM-02 | 64 MiB/document; six historical entries | Bounded retention intent clear; exact sizes/count lack complete product/fixture derivation here | Document object/capacity/lifetime working set and model authority protection; unchanged |
| Shaped text/width cache, MEM-02 | 8 MiB/renderer,32 MiB aggregate; 256 entries,64 KiBkeys,128 KiBpayload/thread | Bounded cache implementation exists; entry counts are not proof of adequate supported text capacity or aggregate footprint | Need populated payload, thread incidence, eviction/recovery and headroom evidence; unchanged |
| Control meshes, MEM-02 | 256 entries,4 MiB CPU/4 MiB GPU per renderer | Bounded-admission intent, but no validated representativeness derivation here | Measure actual consumer/host incidence and pinned references; unchanged |
| GPU geometry/screen, MEM-02 | 64 MiB/document+16 MiB/host; one old + one new generation | Generation bound expresses correctness/lifetime architecture. Byte ceilings need supported fixture evidence | Preserve generation correctness; audit capacity/peak derivation; unchanged |
| Glyph atlas, MEM-02 | 32 MiB/renderer,128 MiB aggregate | Format/extent arithmetic can calculate allocation, not acceptable workload capacity | Measure text/locale/host working sets and eviction, retain caps |
| Terminal graphics, MEM-02 | 64 MiB decoded CPU/64 MiB GPU | Also constrained by existing terminal protocol admission; no waiver from this performance amendment | Reuse terminal authority and memory evidence, enumerate stricter bounds, retain all |
| Upload staging, MEM-02 | 16 MiB/host,64 MiB aggregate | Bounded/chunked-upload architecture clear; limits not independently justified for all consumers | Measure transient/concurrent payloads, preserve chunking/correctness and caps |
| Aggregate GPU, MEM-02 | 512 MiB peak | Admission ceiling, not measured physical residency; A1 sampled~90 MiB does not justify all-host peak/working-set budget | Need aggregate live/retiring accounting and hardware/driver distinction; unchanged |
| R4 optional images/metadata, PM050 | 45 MiB optional; 4 KiB metadata; one current + one retiring | Concrete arithmetic:44.15625 MiB optional at1280×800,150.8125 MiB two complete generations; fixed metadata proof previously recorded. Supports fit for that layout, not broad hardware/product acceptability | Reuse arithmetic/proof at exact scope; audit supported extents/aggregate incidences; unchanged |
| Recovery, MEM-02/REC | CPU 100 ms; visiblep95 250 ms; active 2s; bounded backoff | Bounded failure/recovery intent justified; exact durations lack complete representative calibration here | Preserve bounded-retry/resource/correctness rules; audit timing values and unavailable displayed method |
| Endurance/regression, MEM-03/MET-06/STAT-01 | 60min,200 cycles,20 recoveries; 5%regression; 250 ms idle; threeabsolute/sevenrelativepairs | Observation durations/counts and regression triggers have protocol roles, not guaranteed statistical power or product budget derivation | Preserve current protocol/status; document uncertainty, power/coverage and decision role before future changes |

The audit distinguishes **absent documented validation** from proof a number is
wrong. Existing rejection of bad resize behavior remains an owner observation;
it does not mathematically derive a universal CPU ceiling. A1's diagnostic 64 MiB,
16 MiB numeric,300s/646observation and 0.25 ms/5%selection rules are bounded study
protocol choices, not product acceptance budgets; they cannot become such by reuse.

GBB-P03 must complete evidence citations and scope/feasibility review for any
number it proposes to retain as a newly ratified budget or change. Any unsupported
replacement stays undecided. This audit changes no adjacent numerical threshold.
