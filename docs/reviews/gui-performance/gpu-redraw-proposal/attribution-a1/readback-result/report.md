# A1 readback study disposition

**The study ended without attribution measurements.** The single optimized
build, offline negative controls and default-feature check passed. The sole
native process then failed its scale-one precondition, before wgpu instance,
adapter or device construction. It collected zero GPU observations and zero
conformance, correctness or timing samples. Its64ms process duration is not a
GPU measurement.

The failure was in the new diagnostic launcher. The existing pinned runner sets
`WINIT_X11_SCALE_FACTOR=1` at `scripts/gpu_r3_native_trial.py:58`; the A1 launcher
omitted it and the parent variable was absent. The assertion proves the observed
scale differed from1; the actual value was not recorded. No change in system DPI,
unsupported GPU feature, native COPY_SRC failure, renderer defect or hardware
limit was established. This launch prerequisite should have been carried over
before the approved process was spent.

The original first-failure rule applies. No environment correction/relaunch,
extra GPU process, replacement experiment or optimization was performed. The
launch defect is recorded as `dat-gpu-a1-scale-l0g6`, related to the main GPU
issue, without execution authorization or successor placement.

## Work completed and preserved

The implementation used the shared regional encoder, preparation/upload owners,
production marker resources and native acquisition/presentation owner. It added
only diagnostic controls and an ignored feature-gated native test. The exact
compiled source, logs and predeclared analysis are preserved in `raw.tar.gz` and
`diagnostic-source.patch`; `execution-state.json` and `receipt.json` bind their
hashes and consumed allowance. The patch applies to the recorded source base.

The compiled test executable is retained at
`target/release/deps/datum_gui-2b88501e8cc7fcea`, SHA256
`48b46328d255bc117d30936985c60386fcae75b399da2cce1ab51a5c475f2678`.
There is no authority to run it again from this study's unused observation count.
One optimized build, one scoped Cargo check and one native process were consumed;
the second permitted check was unnecessary after the native stop.

The offline receipt test passed its missing/reversed-query and duplicated/reordered
sample controls. Source health and its13policy regression tests, dependency
and Cargo resource policy checks passed. These prove build and offline readiness
only. Runtime D/P equivalence, marker behavior, per-sample exact output and
resource accounting were never reached and remain unproved.

After preserving the source patch, only this session's diagnostic source edits
were restored. Production source is byte-identical to its pre-study state.
The existing R4 production binary still hashes to
`5c1c8f35746b55397f95280799be3b7bc4f28984b76f94514aaac6ef4933816b`,
matching `r4-native-result/build-state.json`. No diagnostic infrastructure or
common-encoder refactoring was landed into production.

## Consolidated GPU completion assessment

| Category | Current evidence | Effect on completion |
|---|---|---|
| Renderer defects | The previously demonstrated R4 native attachment-capacity defect was corrected. Its preserved native workload completed with exact endpoint output, a visible crosshair and complete input evidence. A1 found no further renderer defect. | Preserve R4. This bounded result is not exhaustive correctness/recovery/endurance qualification. |
| Measurement defects | Original A1 omitted a legal target-readback configuration. Its approved correction compiled, but the new launcher omitted the pinned scale override. | Both are defects in the diagnostic work. Neither can be relabeled as a hardware/backend finding. |
| Measured budget failures | Valid R4 remains p95 6.700167ms/p99 10.205084ms against4/8ms. The earlier corrected candidate remains p95 4.330583ms under its separate method. | No budget pass and no matched ranking. A1 provides no replacement numbers. |
| Attribution and feasibility | No backend-operation residual, marker effect, preparation saving or painter/restoration contrast was measured. | The required-versus-avoidable cost split, feasibility of another strategy and economic value of pursuing4ms remain unknown. |
| Qualification gaps | Complete DRM lifetime/duty observation and broader consumer/backend/scale/recovery/endurance and independent replay remain separate obligations. | S4/S5 and the main GPU issue remain open. CPU optimization remains deferred. |

**Disposition: inconclusive; stop this study.** This work adds no measured
justification for another renderer redesign or optimization campaign. The
requested attribution answer was not obtained. No new experiment is proposed
here, and no successful GPU repair or qualification is claimed.
