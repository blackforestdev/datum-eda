# Bounded zoom no-op correction: proof defined before implementation

Owner direction: `sequencing-owner-direction.json`, point 3. This is a bounded
correction to existing decision-023 camera behavior, not adoption or ratification
of the proposed shared scheduler/cache architecture. The wider implementation
reservation and complete specification approval remain separate.

Contract: `specs/GUI_SHARED_ENGINEERING_CONTRACT.md` E02. A camera operation
that leaves the camera identical must return unchanged and preserve the current
prepared frame. Effective zoom still uses CameraEngine and the existing
pointer-pane/focused-pane routing. No epsilon dead zone is permitted.

Evidence selecting this change: optimized candidate 2874a8e1, disposable DOA2526
board, X11/Xwayland, 1280x800, pointer (420,400), 40 upward wheel events to reach
the zoom limit, then three ten-second 10 Hz upward-wheel trials. Initial trials
performed 178 and 187 prepared-frame rebuilds with identical before/after PNG
hashes, consuming 4.9–5.0% of one CPU core and 10.34–11.70% DRM render-engine
active time. Final raw results are preserved in the measurement report. This
establishes avoidable repeated work; it does not qualify native Wayland UX.
Subsequent reverse controls rejected the initial X11 PNG method as displayed
pixel evidence. The final pair uses validated compositor captures and decoded
RGBA equality; the rejected run remains in the evidence rather than being hidden.

Scope: `crates/gui-app/src/runtime_camera_pane.rs` and its `zoom.rs` child.
Both wheel and focused/menu zoom use one shared changed-state result after
applying unchanged CameraEngine math. No scheduling API, cache lifetime,
rendering quality, world data, dependencies or settings behavior changes.

Required positive proof:

- Unit tests on the production zoom helper: both clamps, unity, rejected
  non-finite/nonpositive deltas, ordinary positive/negative zoom, and a small
  representable zoom change. Compare effective changes with CameraEngine's
  unmodified output, including off-center anchor behavior.
- Guarded GUI test/build, and current shared camera tests.
- Repeat the same native XTest clamp schedule against the optimized candidate:
  zero prepared rebuilds/submissions during stable clamp input; unchanged pixels.
- Repeat ordinary in-range zoom and pointer workloads: effective changes still
  render, final output remains correct, no repeatable >5% unexplained resource
  regression. Record actual schedules, trace overhead and measurement limitations.
- Focused menu/keyboard commands retain their console feedback and menu effects.
  Their caller currently ignores the changed-camera return value; zero total
  submissions is asserted only for wheel-camera input without other UI damage.
- Keep retained-world misses zero during warm camera workloads; returning to
  idle must not start a loop. Existing input-modal Preferences is untouched.

Negative control: the baseline executes the actual production wheel handler
with the unconditional invalidation. Its repeated prepared misses/submissions
fail E02 despite identical pixels. Do not fabricate historical broad HP passes
from this single case. HP06/HP08/HP10/HP24 are affected coverage: current trace
proves retained cache/prepared behavior and native route activity, not byte-level
upload or complete display-opportunity accounting. Those stronger oracles remain
for their adoption slices. HP01–HP25 all remain mandatory at final acceptance.

Rollback: restore only the shared changed-state test and its two call sites;
no data migration is involved. The measurement scripts/receipts remain evidence.
Code passing this bounded proof is not complete renderer acceptance.

## Verification results

The guarded GUI zoom tests passed 3/3; existing shared-camera tests passed 7/7;
the optimized GUI build passed. Exact commands are in `verification.json`.

Three ten-second clamp trials using validated compositor capture recorded:

| Metric | Preserved baseline | Corrected candidate |
|---|---|---|
| CPU, percent of one core | 5.00 / 4.90 / 5.20 | 0.50 / 0.50 / 0.50 |
| Prepared rebuilds / submitted frames | 186 / 182 / 185 | 0 / 0 / 0 |
| GPU render-engine active duty, median | 11.35% | 0% |
| Retained rebuilds and world bundle re-encodes | 0 | 0 |
| Full-window pixels unchanged during clamp | All trials | All trials |
| Same-instance reversed wheel changes displayed output | All trials | All trials |

The initial pair had path-derived object-identity differences. A same-input-path
baseline control establishes exact rendered-model field and central pixel parity
with the candidate, including reversal. Only the non-rendered root board UUID
differs; top/bottom project revision labels are excluded from cross-run image
comparison. See `measurements/matched-model-pixels.json`. No pixel tolerance was
introduced to accept the initial mismatch. The separate import-order issue is
`dat-import-path-painter-order-u6i`.

Matched-import ordinary pointer and in-range zoom trials preserve effective
rendering and report zero retained misses and zero world-bundle rebuilds.
Pointer CPU medians are 12.1% baseline / 10.5% candidate, GPU duty 30.31% / 29.36%;
zoom CPU medians are 6.2% / 5.8%, GPU duty 13.07% / 12.86%. Treat these as a
no-regression check in this sample, not evidence that the no-op correction speeds
up ordinary rendering. Input counts and frame counts differ through scheduling;
neither is a delivered-FPS measurement. Raw repeats are retained in
`measurements/comparison.json` and its linked reports.

All current numbers use optimized X11/Xwayland on the recorded daily machine.
They do not establish native Wayland input, complete presentation/latency,
endurance, larger-scale or all25HP acceptance. GPR-01/02 remain open for the
full specification. This result advances the bounded correction only.

The final same-input-path comparison with timing logs disabled also passes all
three full-window clamp and reverse-wheel controls. Baseline CPU is
5.4 / 5.4 / 5.0% of one core versus 0.5 / 0.5 / 0.5% for the candidate;
baseline GPU render-engine duty is 11.00 / 11.76 / 11.37% versus zero.
Both return to zero measured CPU/GPU activity in the final ten-second idle sample.
Thus the resource improvement survives disabling timing logs. Frame/cache counts
are unavailable in those reports, not zero; the earlier traced trials supply
structural proof. Non-simultaneous samples and CPU-jiffy precision do not yield
an exact instrumentation overhead estimate, and none is claimed.
