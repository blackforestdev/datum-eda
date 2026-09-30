# S4 closure reconciliation and smallest residual scope

<!-- EVIDENCE:GUI-PERFORMANCE-IMPLEMENTATION:S4-CLOSURE-RECONCILED -->
Planning against HEAD `322d54ac` under the owner's explicit S4 priority. Current
GUI app/render source is identical to corrected R4 `c3949e53`; this review changes
no production code and runs no build, GPU test, benchmark or native diagnostic.
`evidence-reconciliation.json` preserves hashes, all 50 carried requirement rows,
the three explicitly S4-assigned historical rows and source continuity.

**Finding:** no demonstrated residual S4 implementation or correctness defect was
found in this reconciliation. The original S4 implementation credit remains
usable, supplemented by the shared RenderSession migration, R4 proof, native
ledger correction and accepted pointer assessment. That is not full product or
S5 acceptance. One explicit contractual exit gate prevents declaring S4 closed:
the reopened S4 paragraph still requires non-withdrawn GPU duty proof. Complete
DRM accounting is absent. The smallest recommended residual scope is an explicit
S4/S5 boundary disposition followed by governance-only S4 closure using existing
proof. It requires no new renderer edits, rebuilds, measurements or optimization.
The proposed boundary change is **not ratified by this planning document**.

## Controlling authority and evidence key

- **IC**: [implementation contract](../../../../specs/GUI_PERFORMANCE_IMPLEMENTATION_CONTRACT.md),
  S1–S5 table, GPI-S4/GPI-S5 and “GPU redraw architecture is an S4 exit prerequisite.”
  Its earlier S1/S2/S3 exit amendments explicitly retain implementation deadlines
  while moving complete per-row qualification to S5.
- **SE**: [shared engineering contract](../../../../specs/GUI_SHARED_ENGINEERING_CONTRACT.md),
  E02/E04, E06/E07/E09 and SCH/REC; **AM**:
  [acceptance matrix](../../../../specs/GUI_PERFORMANCE_ACCEPTANCE_MATRIX.md),
  MEM-02, GPU-01–03, ACC-01/02, MET-03/04 and STAT-01.
- **PM049/050**: [shared render session](../../../decisions/PRODUCT_MECHANICS_049_SHARED_RENDER_SESSION.md)
  and [damaged-sample/regional composition](../../../decisions/PRODUCT_MECHANICS_050_DAMAGED_SAMPLE_RESTORATION.md).
  PM051 withdraws GPU 4/8 ms gates only, not adjacent limits or correctness.
- **E1**: [historical S4 readiness](../implementation/S4/readiness/result.json),
  source 48d99c50, and [guarded native layout](../implementation/S4/guarded-native-layout/result.json).
  Reopening for GPU redraw did not invalidate those implementation credits.
- **E2**: [shared source contract](../gpu-redraw-proposal/implementation-source-contract/receipt.json),
  [terminal lease](../gpu-redraw-proposal/implementation-terminal-lease/receipt.json),
  [preparation profiles](../gpu-redraw-proposal/implementation-preparation-profile/receipt.json),
  [normalized graph](../gpu-redraw-proposal/implementation-normalized-graph/receipt.json),
  [shared native host proof](../gpu-redraw-proposal/implementation-prefix-images/native.log).
- **E3**: [R4 GPU proof](../gpu-redraw-proposal/r4-gpu-proof/receipt.json) at 544dc61a;
  [proof preparation](../gpu-redraw-proposal/r4-proof-preparation/receipt.json) for
  metadata, method pins, negative controls and scope exclusions.
- **E4**: correction commit c3949e53 (23 native queue/ledger tests, including
  ten-entry union retirement and five-image device loss), followed by the
  [first valid R4 native result](../gpu-redraw-proposal/r4-native-result/receipt.json).
- **E5**: [pointer assessment](../baseline-budget/pointer-report.md),
  [raw-evidence receipt](../baseline-budget/pointer-receipt.json) and
  [exact owner acceptance](../baseline-budget/pointer-owner-disposition.json).
  Three GPU trials including the reused archive, three quiet trials, zero new
  failed/replaced trials; owner accepts this limited assessment only.

## Finite S4 implementation/correctness checklist

Each row credits bounded implementation evidence, not all-consumer acceptance.

| ID | Controlling requirement | Committed implementation and existing proof credited | S4 disposition |
|---|---|---|---|
| C01 | IC S4 / GPI-S4; SE E02, HP09-01/02: preserve PTY/state, suppress hidden render work, restore consumed damage on refusal | `runtime_terminal_render.rs` bypasses closed-dock snapshots. `terminal_session_render.rs` restores damage from earlier split leaves on later failure; `terminal_core_adapter.rs` bounds merged damage. E1 native hidden-output/reopen proof and E2 terminal-lease tests (26 render, 15 registry, 13 adapter) plus actual shared native host test | Implementation credited. Full mixed-throughput/byte/native qualification stays S5; do not infer it from zero hidden snapshots |
| C02 | IC S4 fairness; SE SCH and S4 table | Existing shared native-round terminal drain and fair host scheduling, 18 coordinator tests including alternating first service; four-host native component proof in map `s1_active_host_fairness_work`; E1 readiness credits this | No new scheduling implementation needed. Saturated multi-session latency/throughput and numerical qualification stay S5; CPU event-loop optimization not reopened |
| C03 | IC S4 all-host recovery; SE REC | `native_device_recovery.rs` stages all hosts before committing, preserves old state on staging failure, transfers same-host CPU caches only at successful commit, and readmits shared restoration. E1 staged-failure proof plus S5 `device-cpu-cache-retention/result.json` affected GPU proof | Adoption credited. All-config fault, CPU/time/resource and endurance qualification stays S5; offscreen proof is not a native multi-host failure replay |
| C04 | IC S4 two-generation bounds/serial multiwindow; SE E06/E07, E09 HP25-01; AM MEM-REPLACEMENT | Unchanged glyph/terminal generation owners retain submitted resources. E1 replacement proofs; E2 actual `native_shared_device_hosts_preserve_hidden_terminal_and_console_work` passes on shared Main/Global/Project/New device | Bounded implementation credited. Separate concurrent multi-device issue `dat-gpu-test-concurrency-6dt` remains outside this serial prerequisite |
| C05 | IC S2→S4 carry: accounting/caps, exact changed ranges, shaping/layout dependencies; AM MEM/ACC; PM047 private-allocation boundary | All 50 `s2_exit_disposition` rows have empty `unfinished_implementation`; 27 completed at S4, 23 without a newly asserted implementation defect. E1, recorded per-row artifacts and subsequent guarded allocation/source changes are credited individually in `evidence-reconciliation.json` | No recorded S4 implementation gap. All 50 original qualification predicates remain visible at S5; none is promoted to passed |
| C06 | PM049 shared ownership and retirement of competing paths; IC GPU redraw amendment | `RenderSession` owns retained/prepared content, dependency invalidation, damage and completion. Main and auxiliary hosts call shared prepare/plan/encode/present contracts. Legacy Runtime prepared/retained/terminal cache fields and public raw rendering bypasses removed. E2 typed-input, profiles, normalized encoding, offscreen and native adapter proof | Shared adoption credited. Native acquire/present/device completion remain necessary platform responsibilities, not legacy defects |
| C07 | PM050 r4 warm graph; IC production GPU redraw correction | Whole selected 32px tiles are restored and repainted through ordered suffix traversal; bounded atlas hardware resolve updates C, then complete C is copied to every acquired target. Empty warm damage copies C only. Full-size B warm restore/render/resolve path superseded; cold/full fallback retains required B | Mechanism implemented, E3 and E4/E5 native operation credited. Less pixel work is not asserted to imply lower cost or a hardware floor |
| C08 | PM050 exact 8x, coordinates/clips, painter order and invalidation/recovery | E3 compares all eight samples against independent full painter, fractional geometry/glyphs, all 256 atlas cells, seams/edge padding, Console/terminal/text/menu, wrong-origin/sample and duplicate/missing-paint negatives. Old presented damage, coalescing, failed completion, material-hover invalidation and overflow fallback tested | Bounded correctness credited. No tolerance/golden relaxation; wider backend/scale/consumer proof remains S5 |
| C09 | IC affected live/retiring accounting; PM050 optional 45MiB/metadata 4KiB/aggregate 512MiB, one current+retiring bundle; AM MEM-02/ACC | E3 five-image admission/refusal/retirement; 44.15625MiB optional and 150.8125MiB two-generation payload at reference extent; 3936-byte bounded image metadata proof. E4 publishes shared ten-entry usage capacity to native ledger and verifies complete membership/last-use/close/device-loss | Demonstrated native ledger defect corrected. API capacity/lifetime implementation credited; opaque driver residency and full-process/native peak qualification are not fabricated |
| C10 | IC bounded F-DOA/P630/X11/1x pointer proof, exact input/final output | E4 first valid native run and E5 five further valid runs on unchanged binary; all 1280 integer pointer changes from 3600 requests completed, independent readiness/final pixels exact, visible crosshair, normal drain/exit, unchanged authored fixture | Required bounded native correctness/stability credited. No repeat justified merely by handoff; continuous displayed latency/endurance remain unqualified |
| C11 | IC MET/GPU/ACC/STAT for claims; PM050 complete timing | E3 actual cold/warm/empty/fallback encoder paths include final copies; worst cold graph 31/32 queries. E4/E5 complete native demand/submission/drain validation. Archived nearest-rank tails independently reproduced | Timing claims credited at scope. E5 measures timestamp-mode CPU effects only; total observer effects belong GBB-P05 and S5 method qualification |
| C12 | IC explain material CPU regressions without CPU optimization | E5 paired timestamp-mode CPU increments ~1.31s/30s explained as query/poll/log mode effect with common semantic observer, not a matched renderer-before/after regression. A1 preparation bypass ~0.025ms at diagnostic scope, not native CPU acceptance | No demonstrated unexplained production CPU regression is established by these comparisons. Clean CPU/resource numerical qualification remains S5; neither a pass nor a waiver is claimed |
| C13 | IC reopened S4 exit paragraph expressly retains engine duty ≤25% and says missing conformance/non-withdrawn numerical proof cannot pass | Complete DRM client/context lifetime accounting is absent. Own-queue timestamps, successful drain and held descriptors do not establish complete duty. PM050 explicitly leaves this method separately unapproved | **Current contractual blocker, not a demonstrated renderer defect.** Requires the explicit boundary disposition below or a separately authorized, feasible duty method; no duty pass claimed |

The old historical Console 3778-pixel failure is not an outstanding S4 blocker:
`implementation/S5/console-reconciliation/result.json` records corrected static
references and ten exact scale cases, nine Console tests. Preserve that old failure,
credit the superseding proof, and retain broader Console native qualification at
S5. Likewise the old three-slot attachment panic is corrected by E4; it must not
be carried forward as an open renderer defect. Withdrawn 4/8 ms exceedances and
A1's inconclusive attribution are not new S4 defects or reasons to optimize.

## Source continuity and evidence limits

Nine of 21 historical readiness owner hashes are unchanged. The twelve changed
owners were reviewed, not treated as automatically invalidating all old evidence:

- Main/auxiliary preparation and native hit tests migrated to shared sessions;
  terminal split rollback and bounded merge gained E2 focused tests and the
  actual native host proof. No terminal core/PTY replacement occurred.
- Device recovery added bounded diagnostic requests and commit-time CPU cache
  retention. Existing S5 recovery/cache records preserve both successes and
  numerical failures; do not promote them to complete recovery acceptance.
- Allocation lifetime/consumer owners gained trace/event serialization and
  attribution. CPU/private-call code gained observed realloc overlap accounting
  and guarded refusal fixes. `S5/scoped-reallocation-peak/result.json` and
  `S5/private-allocation-lifetimes/result.json` cover affected behavior at scope;
  complete memory/observer qualification remains open under PM047 and GPI-S5.
- Measurement-owner changes update shared-session test adapters and attribution;
  text-buffer areas now carry painter layer identity, covered by E3 all-painter
  proof. Shaping/measurement key owners and generation bounds retain their
  unchanged ownership or explicitly recorded migration evidence.

The only three GUI files changed after E3 are the attachment-capacity bridge,
its native queue tests and shared capacity declaration; E4 covers that delta.
No GUI source changed after c3949e53 through this review's base HEAD. Source
continuity plus affected existing proof supports reuse; it does not assert every
possible regression is disproven. The audit found no demonstrated residual code
bug; it is not a substitute for final independent S5 replay.

## Qualification retained at S5

| ID | Obligation and controlling allocation | Credited evidence / explicit remainder |
|---|---|---|
| Q01 | IC S1 exit and S5: all carried SH/LF/affected-HP predicates, prescribed negatives, native input/focus/final state/static coverage | Keep the original 30 S1 rows and acceptance rules. Existing adapter proof is credited; missing backend/scale/negative coverage remains individual, not new S4 code work |
| Q02 | IC S2 exit and S5: all 50 original S2 qualification rows | Row-by-row `qualification_predicate` in evidence JSON preserves MEM/ACC/reuse/painter criteria and original artifacts; zero remaining recorded implementation gaps does not mean numerical acceptance |
| Q03 | IC S3 exit and S5: 20 carried S3 qualification rows | Preserve per-consumer control/scroll/clip/hit/focus, cache and forbidden-work negatives, native appearance and backend/scale evidence |
| Q04 | GPI-S5, AM MEM/GPU/ACC/STAT: complete native/resource/method conformance and non-withdrawn numerical qualification | Driver/DRM lifetimes, process peaks, observer effects and admitted configurations remain unqualified. Duty is technically qualification work, but C13 currently also gates S4; the proposed amendment must ratify exclusive S5 placement |
| Q05 | IC S5: admitted daily backends/scales, 60min/200-cycle/20-recovery contract | Credit existing 60-minute and 20-recovery evidence at original pins, including recorded failures; do not restart the entire campaign by assumption or claim complete acceptance |
| Q06 | IC S5: distinct-reviewer native replay and separate owner UX/product disposition | R4 independent design review is not native replay or product ratification; accepted pointer assessment is only its expressly limited scope |
| Q07 | IC scope exclusions; SE E09 HP25 and resize amendment | Concurrent multi-device failure remains separate; resize resource/temporal, temporal/T2 and unadmitted schematic limits remain explicit. Do not invent new S4 implementation obligations from excluded scope |

## GBB-P05 retained separately

The owner's accepted-undecided disposition schedules seven workload families
(idle, pan, zoom, Preferences scroll/controls, pane transitions, native lifecycle,
mixed interaction), total instrumentation effects, displayed responsiveness,
supported-hardware expectations and complete DRM/resource accounting. Its purpose
is representative baseline/budget rationale; replacement budgets remain undecided.
No additional measurement or renderer optimization was authorized.

DRM/resource and method evidence can serve both S5 qualification and GBB-P05;
these are two uses of one missing proof, not two separate campaigns. S5 owns
qualification under retained requirements; GBB-P05 owns baseline/budget derivation
and provenance review. The planning of either lane does not authorize execution
or block functionality globally.

## Smallest residual closure packet and decision

<!-- OWNER:GUI-PERFORMANCE-IMPLEMENTATION:GPI-S4-CLOSURE-DISPOSITION:BOUNDARY -->
**Requested owner disposition:** approve the following narrow S4/S5 boundary
amendment and governance-only S4 closure, or retain C13 and request a separately
scoped method proposal. No production or runtime execution is proposed.

Proposed controlling amendment (not effective until approval):

> S4 closes on shared-renderer implementation and the affected bounded
> correctness, invalidation/recovery and application-owned resource admission/
> lifetime proof credited by checklist C01–C12. Complete DRM/client duty,
> instantaneous/peak resource accounting and cross-configuration measurement
> conformance qualify the unchanged numerical requirements at S5; they are not
> prerequisites to the S4 implementation exit. The 25% engine-duty criterion is
> neither changed nor declared met. All correctness and application-owned
> admission/lifetime bounds remain required at S4. GPU 4/8 ms limits remain
> withdrawn. GBB-P05 remains a separate nonblocking baseline/budget lane.

Finite residual checklist:

1. **Current blocker C13:** owner ratifies that exact boundary amendment. The
   authority is IC's reopened S4 exit paragraph; this planning review cannot
   silently waive its explicit 25% gate. If retained, complete DRM method
   feasibility is still unknown, and no honest finite renderer patch or replay
   can be promised to satisfy it.
2. **Governance proof/closure only after approval:** record the ratified wording
   in the controlling contract/decision, verify the current GUI tree and credited
   artifacts still match this review, reconcile the adoption map and mark only
   GPI-S4 complete. Run the applicable non-compiling state/governance checks.
   **Production files: none. Builds: zero. New GPU/native/performance runs: zero.**
3. **Explicit handoff:** retain every Q01–Q07 and GBB-P05 gap, the open main issue,
   and the separate concurrent-device/resize exclusions. Do not auto-authorize an
   S5 campaign or infer product acceptance from S4 closure; selection/execution
   scope for S5 or functionality must be explicit.

There is no recommended residual renderer implementation packet because this
review found no demonstrated implementation gap to repair. If a new code defect
is later demonstrated, tie it to its exact requirement and scope a correction;
do not manufacture one to justify a build. S4 is not declared closed here.
