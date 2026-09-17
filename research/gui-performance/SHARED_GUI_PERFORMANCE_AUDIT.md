# Shared GUI performance audit: planning evidence

Recorded 2026-09-17. Static audit baseline: a902ed99. No fresh runtime benchmark
was performed in this audit. Later specification work must reconcile current
source and the owner's environment before treating these findings as current.

## Owner intent and limits

The owner requires excellent visual rendering and excellent scrolling, zooming,
opening/closing and pane UX with the least practical CPU/GPU/memory consumption.
The accepted amendment schedules specification development first, preserves
bounded concurrent Preferences QA, and separately fences implementation.
The owner selected shared-foundation scope, daily Linux setup qualification,
60 Hz baseline, and a narrow read-only research exception. Approval is for the
roadmap/planning amendment, not a mechanism decision or numeric budget ratification.

## Code observations

| Observation at baseline | Evidence | Interpretation/limit |
|---|---|---|
| General redraw helper invalidates multiple owned windows | crates/gui-app/src/app_shell.rs, request_redraw_if_needed | Audit each caller's actual affected consumers; not every multiwindow update is unnecessary |
| Camera handlers invalidate the prepared frame | crates/gui-app/src/runtime_camera_pane.rs and interaction_refresh.rs | World geometry survives, but unrelated prepared shell content may be rebuilt |
| Frame preparation builds shell, menus, panels and pane content | crates/gui-render/src/render/scene.rs | Refine ownership before deciding what is safe to retain |
| Screen-space buffers are uploaded on general rendering | crates/gui-render/src/render/gpu_vertex_upload.rs and gpu_data.rs | Buffer capacity reuse alone does not eliminate unchanged-byte copies |
| New Project clones workspace and builds retained world content | crates/gui-app/src/global_preferences_window.rs, new_project branch | Preferences already has a dialog-only path; New Project needs the same ownership boundary |
| Stroke upload identity uses pointer and length | crates/gui-render/src/render/gpu_strokes.rs; render/types.rs owns a Vec | Investigate lifetime-safe identity; this audit did not reproduce a stale-frame failure |
| Pane-operation cache test simulates reuse | crates/gui-render/src/render/tests.rs, pane_ops_do_not_re_resolve_the_world_scene | It does not prove every production event handler avoids invalidation |
| Preferences has shared continuous scrolling; Layers has row scrolling | crates/gui-viewport/src/scroll.rs; crates/gui-render/src/side_panels/layer_scroll.rs | Share compatible machinery without erasing legitimate control semantics |

Existing shared assets include Renderer, retained board/schematic geometry,
render bundles, text caches, camera/grid/hit/interaction tooling, and event-loop
sleep/presentation notifications. This is not evidence that every feature builds
its own renderer, nor evidence that existing doctrine is enforced consistently.

## Historical measurements, not a new baseline

- 7bc8a701 records sidebar pointer CPU improvement from 36.53% to 5.60% of one
  core after layout caching; board-hover cost remained unresolved in that sample.
- c08d1ff7 records controlled native Wayland zoom CPU 24.67% to 13.13%, with
  retained draws/geometry; handler input was not physical wheel input.
- f535b8ae records Project Preferences manual scrolling CPU 93.43% to 22.87%
  after eliminating hidden workspace preparation; manual rates were not pinned.
- 579adb6a adds bounded dialog text/width caching and single-pass rendering.
- 4965d758 records 6.7647% of one CPU core, 1.6906% DRM render-engine active time,
  193 submissions in about 30 seconds, and 3751 microseconds median renderer time.
  Submission rate is not delivered FPS, and GPU active time is not power usage.
- a902ed99 adds fractional scrolling, clipping and functional scrollbar geometry;
  its owner feedback is "Responsive; scrollbar tracks correctly". It does not
  establish new CPU/GPU qualification. Concurrent GPU tests crashed; serial
  replay passed; dat-gpu-test-concurrency-6dt retains the unresolved cause.

These samples used different interactions/builds/environments. Do not combine
them into a single apples-to-apples performance series. Existing tests and pixel
proofs are useful evidence but do not establish end-to-end responsiveness.

## Prior art and interpretation

- [Qt Quick renderer](https://doc.qt.io/qt-6/qtquick-visualcanvas-scenegraph-renderer.html)
  describes batching and GPU geometry retention as core strategies.
- [Chromium rendering critical path](https://www.chromium.org/developers/the-rendering-critical-path/)
  separates layout, paint and compositing; different changes can skip stages.
- [wgpu 28 Queue](https://docs.rs/wgpu/28.0.0/wgpu/struct.Queue.html#method.write_buffer)
  documents immediate staging copies and native staging allocations for writes.
- [winit 0.30 Window](https://docs.rs/winit/0.30.13/winit/window/struct.Window.html#method.pre_present_notify)
  documents compositor notification and Wayland redraw scheduling.

Inference for Datum: inspect the whole event-to-presentation path, retain work
by ownership, and qualify resource use alongside visual/interaction quality.
These references are behavioral research, not dependency adoption authority.

The specification phase must read complete existing rendering, typography,
selection, viewport and GUI evidence routes. This initial audit does not claim
that the prior research completely addressed performance or was fully implemented.

## Owner-directed historical coverage amendment — 2026-09-17

A retrospective review compared all six recent corrective commits against the
32-requirement planning draft. It found correct overall direction but missing
explicit encoding/pass obligations, exact scroll input/geometry oracles,
individual defect-sensitive regressions and concrete retirement of inconsistent
consumer paths. The owner requested upgrading the specification so it would
prevent acceptance of every recently corrected faulty behavior.

The updated plan adds R33-R38 and mandatory HP01-HP25. These cover immutable
menu/selection preparation, pointer layout, presentation notification, encoded
world draws, batching and upload identity, hidden consumer work, window-local
redraw, dialog-only preparation, pass/resolve count, text/measurement/control
reuse, glyph-cache transitions, fractional scrolling and complete native input,
scrollbar/clipping/focus behavior, misleading low-duty acceptance and the
unresolved concurrent-test crash. Existing successful mechanisms are reusable
evidence; neither replacement of wgpu nor removal of working fixes follows
from this audit.

The matrix requires a case for each behavior-changing corrective hunk and
reviewed disposition of non-performance changes. First-parent revisions below
identify historical negative candidates; compatibility and fixture/toolchain
closure must be checked before any separately authorized replay. No replay,
new benchmark, independent review or current qualification occurred here.

| Corrective revision | First-parent negative candidate | Required cases |
|---|---|---|
| `7bc8a701d3b7d5bb48d57459efd71ef23314b464` | `6c1e2e1ec1747ac39af062636fbc5b96f1783d24` | HP01-HP02 |
| `5f60ef51ec8e2e6084e12f91d33fff2a6a3fb287` | `7bc8a701d3b7d5bb48d57459efd71ef23314b464` | HP03-HP05 |
| `c08d1ff7d42c0eff8c2b5cd83f3a50e6d42ddff0` | `3029ebd2e33d7b1d694970d820234e617b627f36` | HP06-HP10 |
| `f535b8aec5c0659b92ae636eba7e987a09cbd4aa` | `c08d1ff7d42c0eff8c2b5cd83f3a50e6d42ddff0` | HP05, HP11-HP12 |
| `579adb6a06f74842cd3f6fd22829660175f5261f` | `f535b8aec5c0659b92ae636eba7e987a09cbd4aa` | HP13-HP17 |
| `a902ed99bb14dda8490bc4c385686962d846d4a7` | `4965d75899b295fe53224bc296e05335162dd864` | HP18-HP25 |

Measurement record `4965d758` supplies HP24's acceptance counterexample; it is
not a new implementation defect candidate. HP25 preserves the concurrent GPU
test failure rather than pretending the scrolling repair resolved its cause.
The clause inventory now has 63 unique records: 38 R requirements and 25 HP
cases. The full route was reviewed with the unchanged promotion-owner-direction
record; the latter continues to authorize only its dated planning disposition.
All six specification steps remain pending; numeric budgets and renderer
mechanisms remain proposed, and implementation retains its separate gate.
