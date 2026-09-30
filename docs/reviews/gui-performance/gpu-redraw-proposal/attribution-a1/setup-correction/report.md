# A1 workload initialization: requested setup proof passed

The common diagnostic initializer now explicitly selects the single Board
layout and Editor0 focus. A setup-only path uses that same initializer, captures
old/new images and prepared-state evidence, and returns before conformance or
sampling. No renderer strategy, control or timing algorithm changed. **Zero new
timing samples were collected.** All prior560 measurements remain invalid for
the intended workload and are preserved unchanged.

The [independent review](independent-review.json) found no blocker to this narrow
setup correction after inspecting source, configuration, runtime state, all four
images and independently archived evidence.

## Independent evidence and observed state

The original R4 native input receipt was extracted from its existing archive;
its SHA-256 remains `2056f8c34abb3cfbfac38a268f1e29aa2016d6a334a2ffdbed68c4aaa65537f5`.
Its actual sequences409/1048, not only copied diagnostic constants, were compared
with the new receipts. Original native launch evidence specifies single layout,
1280×800 and scale1; final state pins selectionNone and Editor0 focus.

| Property | Observed and compared |
|---|---|
| Layout | One Board pane0; frame `[224,33,760,709]`; scene `[240,75,728,651]` |
| Focus and selection | `Editor(PaneId(0))`, focused pane0, `None` |
| Camera | `(151634208,95122896)` nm, zoom1, before and after both transitions |
| H, sequence409 | `(499,150)` → `(500,150)`; zone `bddd9739-252d-5af8-baf9-eec0be6b999d` |
| V, sequence1048 | `(700,269)` → `(700,270)`; zone `b3b2418a-f7a4-5848-aa64-eace29e3cb3a` |
| Pointer/hover proof | Actual prepared pointers lie inside board; shared hit-testing independently resolves the pinned zone at all four positions |
| Crosshair | FullViewport; both arms visible at every before/after position |

Camera fields are f32. Rust's shortest round-trip display emits151634200 and
95122900; interpretation at that source type yields exactly151634208 and
95122896. The [offline comparison](comparison.json) retains these exact values;
it uses no epsilon or camera adjustment.

The independently archived endpoint PNG at SHA
`7a66641915cfad691783888a9a1685ebe9783a0e4432b0199e6525016890f622`
anchors the single-pane geometry and board outline. The comparison also checks
four separated crosshair-arm pixels in every new image against the archived
crosshair color, and checks adjacent background pixels. All four images were
visually inspected by the primary session and independent reviewer.

- H: [before](case-0-before.png), [after](case-0-after.png).
- V: [before](case-1-before.png), [after](case-1-after.png).

These full-reference images demonstrate initialized scenes. They are **not**
a new proof of acquired native regional output. The archived PNG is an endpoint
at a different pointer/hover, not an exact H/V raster oracle. In addition to
those intentional differences, the diagnostic shows terminal label `terminal`
instead of `shell1`, and DRC68 status absent from the archived endpoint.
Accordingly this proof establishes the specifically requested layout, camera,
selection, hover, pointer and crosshair state—not full archived raster identity
or equivalence of every workload/chrome responsibility.

## Reproducibility and boundaries

The corrected executable is preserved outside Cargo at the path in
[declaration.json](declaration.json), SHA
`dac64a3f765f795d4cc161a2d228ed2b23a834663f82ec25ee6be0e7a00f91b8`.
The full source patch, narrow [initialization/proof diff](initialization-and-proof.patch),
launcher and configuration are preserved alongside it. The setup flag is
`DATUM_A1_SETUP_ONLY=1`; the branch returns before all conformance and measurement
loops. Output contains two setup-case records and a completion record declaring
zero samples. One setup process exited0 in2.221s with16 counted submissions.
Original process/resource caps remain; no DRM/resource acceptance is inferred.

The first setup-proof build had an evidence-helper serialization compile error.
It was corrected without adding dependencies; the second guarded offline build
passed. Both build logs remain in [raw.tar.gz](raw.tar.gz). The original
preflight's inherited `prior_checks` wording is preserved, with its scope
clarified in [receipt.json](receipt.json): this executable had a new successful
build, while prior offline/default checks are reused only for unchanged code.

After sealing evidence, only this session's temporary diagnostic source edits
were restored. Production R4 source/binary are unchanged; the corrected
diagnostic is retained as executable and source artifacts. Earlier invalid
measurements, original failed launch and missing-artifact history remain intact.

This completes the requested initialization correction and independent setup
review. It neither replaces invalid timings nor authorizes more samples.
Attribution and hardware/economic feasibility remain unanswered; valid prior
R4 budget failures and separate full DRM/S4/S5 qualification gaps are unchanged.
