# A1 disposition: measured batch rejected for incorrect workload

A1 reached the GPU and completed its entire fixed batch, but it cannot answer
the approved attribution question. The diagnostic initialized the default split
Board/Schematic workspace instead of the pinned single-board workspace. Its
exact-output oracle was generated from that same incorrect state. This is a
substantive workload-validity failure in the new diagnostic, not a missing
artifact, environment problem, renderer defect or hardware limitation.

I missed this state-initialization omission during source preflight. Post-run
inspection of both reference images exposed it. No measurements are discarded,
and no replacement batch was run. The 560-measurement allowance is consumed.

## Setup recovery and execution

The preserved 21-file source patch was restored with every source hash matching.
One guarded offline optimized rebuild succeeded in 7m03s without compiler
diagnostics. Its executable SHA-256 is exactly the original
`48b46328d255bc117d30936985c60386fcae75b399da2cce1ab51a5c475f2678`.
The executable is preserved read-only outside Cargo's target tree at:

`/home/bfadmin/Documents/datum-eda-evidence/attribution-a1/48b46328d255bc117d30936985c60386fcae75b399da2cce1ab51a5c475f2678/datum-a1-diagnostic`

That directory also holds the source archive/patch, complete launcher,
preflight configuration, declaration, approved inputs, original packet,
readback correction and all new raw evidence. The executable hash was checked
before and after execution. Temporary diagnostic source edits were restored;
production R4 source and binary remain unchanged.

The rebuilt binary's offline negative controls passed. Preflight verified the
CLI, raw/normalized board and reference hashes, original failure archive hashes,
Mesa version, X11 display socket/authentication, absence of competing compiler
or diagnostic processes, and fixed limits. The launcher sets
`WINIT_X11_SCALE_FACTOR=1`, X11 and Vulkan, clears inherited Datum flags,
Wayland selection and loader injection, and uses isolated XDG directories.
No driver overrides were present. The launcher and configuration were preserved
before launch. Runtime confirmed P630/Vulkan/Mesa25.0.7, required timestamp and
exact8 features, and native COPY_SRC/COPY_DST/RENDER_ATTACHMENT usage.

One native process exited0 in16.207241s. It emitted exactly16 conformance,
14 correctness,56 warmup and560 measured observations. Its submission counter
ended at1941, below2200. Maximum sampled tracked GPU ownership was90,098,524bytes;
the64MiB diagnostic CPU guard passed (no numeric peak was emitted). Numeric output
was1,524,073bytes. These are the diagnostic's bounded checks, not full DRM or
independently audited device-wide lifetime/peak qualification.

All630 sample byte comparisons passed against their internal reference/hybrid,
and conformance completed. The unchanged analyzer verified exact sample order,
counts and bounded nonreversed timestamp records. Its `complete` status means
that mechanical validation passed; this report's subsequent fixture rejection
controls the study disposition.

## Concrete failure and source trace

The pinned `inputs.json` requires `layout: single`. The historical reference has
one board pane spanning roughly x224..984. New [H](reference-0.png) and
[V](reference-1.png) reference images instead show a board pane ending around
x603 with a Schematic pane beside it. H has a shorter crosshair and different
scene geometry; V's pointer at(700,270) lies outside the board pane and has no
visible crosshair. These are not the approved pointer transitions in their
pinned layout, even though the numerical pointer coordinates were copied.

The preserved `gui-app/src/gpu_attribution.rs` calls
`load_board_editor_workspace_state`. `WorkspaceUiState::new` uses
`WorkspaceLayout::default`, which returns `board_schematic` in
`crates/gui-protocol/src/workspace_layout.rs:305`. The preserved
`gui-render/src/render/gpu_attribution.rs::attribution_case` clones this state and
sets selection, hover, cursor and crosshair style, but never assigns the pinned
layout. Normal `app_bootstrap.rs::apply_initial_layout` applies the single preset;
the ignored diagnostic test bypasses that bootstrap. Passing normal application
CLI flags to the Rust test harness would not fix it.

The reference and measured graphs share this erroneous initialization. Their
exact agreement cannot establish fixture identity. Neither the81-pad check nor
the upload/command equality assertions independently verifies the workspace
layout or that both pointer positions are within the board pane. Filed as
`dat-gpu-a1-fixture-py2u`. The earlier scale launcher defect is resolved at its
narrow scope; the main GPU issue remains open.

## Preserved measurements — invalid for approved-workload attribution

The following values describe only the wrong-layout process. They are retained
for audit, not promoted into claims about required operations, avoidable Datum
cost, the original native tail, or whether optimization is worthwhile. Full five
block medians/ranges, maxima and phase timings remain in [analysis.json](analysis.json);
all raw records are in [raw.tar.gz](raw.tar.gz).

| GPU interval median, ms | H-labelled case | V-labelled case |
|---|---:|---:|
| D: resident graph, outer queries | 4.229125 | 2.464083 |
| M: additional production markers | 4.311792 | 2.545542 |
| P: frozen preparation | 4.226250 | 2.471958 |
| A: suffix draws omitted | 2.461083 | 1.708000 |
| B: restoration also omitted | 1.471375 | 1.129708 |
| C: copies only | 0.989333 | 0.754958 |

| Predeclared median block difference, ms | H-labelled | V-labelled |
|---|---:|---:|
| D−N completion wall | 0.314062 | 0.074768 |
| M−D GPU | 0.097042 | 0.087917 |
| M−D completion wall | 0.531362 | 0.142447 |
| D−P preparation wall | 0.023149 | 0.012125 |
| D−A GPU | 1.755958 | 0.747958 |
| A−B GPU | 0.987708 | 0.594042 |
| B−C GPU | 0.483167 | 0.364500 |

The mechanical materiality rule selected D−A, A−B and B−C in both incorrectly
initialized cases, plus D−N completion in H only. M−D completion changed sign
across H blocks; neither M−D nor D−P met the materiality rule in both cases.
These are conditional observations of the rejected fixture, not recommendations.
Even absent the fixture failure, common reset, diagnostic COPY_SRC and serialized
readback would prevent treating these as unavoidable hardware floors or additive
shares of historical native p95. No overhead is subtracted and no percentile
budget is declared passed.

## End-to-end GPU assessment and stopping decision

- **Renderer defects:** prior shared-renderer and R4 corrections retain their
  existing proof. This run establishes a diagnostic fixture defect; it does not
  establish another legacy renderer fault.
- **Measured budget failures:** the valid prior R4 workload remains p95
  6.700167ms and p99 10.205084ms against4/8ms requirements. The earlier candidate's
  p95 4.330583ms remains evidence at its own method, not a matched ranking.
  This wrong-layout run changes neither result.
- **Qualification gaps:** full DRM lifetime/duty, wider consumer/backend/scale
  and recovery/endurance coverage and independent replay remain separate. A1
  neither attempted nor satisfies them. CPU optimization remains deferred.
- **Feasibility and value:** this batch cannot quantify approved-workload backend
  cost versus avoidable implementation overhead or resolve marker effects on that
  workload. It supplies no sound basis for another redesign or optimization
  campaign, nor proof that the host cannot meet4ms.

Stop here under the substantive-failure rule. Correct attribution requires the
pinned single-layout state and an independent pre-sampling fixture check; that
would require another measured batch, which the unchanged maximum prohibits.
No further run or renderer change is made or proposed for automatic execution.
The diagnostic consumed its allowance without producing valid approved-workload
attribution. S4/S5 and `dat-gui-performance-implementation-vkq` stay open.
