# Painter/hover correction: valid pointer result, p95 still fails

The shared correction removed the observed cold redraws and preserved exact
output, but it did not meet the complete GPU p95 budget. The campaign stopped
at C-G1, its first trial. No baseline, quiet, replacement or later trial ran.
Candidate source is `453cd9f2`; declaration, raw evidence and analysis are pinned
by `receipt.json`. The successful four-group GPU proof and single release build
are recorded in the linked prerequisite receipts. The baseline was reused.

| Complete active-frame GPU span | Result | Required limit |
|---|---:|---:|
| p95 | 4.330583 ms | <=4 ms: failed |
| p99 | 5.701750 ms | <=8 ms: within limit in this trial |
| Maximum | 5.964167 ms | Descriptive |

All 1,277 active frames used upload-leading/restoration/suffix passes, with zero
world submissions. The same 14 non-pad hover transitions from the prior failed
trial occurred; none caused the old world upload/full-prefix/copy graph. The
observer held 2,573 of 4,096 records without overflow. All 1,281 GPU frames remain in the complete evidence: 1,277 active, three
startup and one close frame.

The workload sent 3,600 requests, of which 1,280 changed integer position; all
1,280 changes were received and completed their viewport route. Readiness and
final images each had zero changed pixels against the pinned reference. The
final image was inspected and its full crosshair is visible. Shutdown drained,
exited normally, left no processes and required no forced cleanup. The binary
hash remained unchanged. This is a valid numerical failure, not another
instrumentation failure.

## Remaining measured cost and source assessment

Restore-pass p95 was 0.198833 ms; suffix-pass p95 was 3.925250 ms. The complete
span also includes uploads and intervals outside pass timestamps. These are
separate distributions; their quantiles must not be added or subtracted.
846 of 1,277 active spans exceeded 4 ms, and none exceeded 8 ms. Retaining all
samples is essential: this is not a single removable outlier.

The measured non-pad invalidation defect is corrected in the shared owner.
No remaining legacy-editor bypass was demonstrated by this run. Source review
identifies two kinds of work still present in the shared suffix:

- `gpu_frame.rs` performs one full-target hardware resolve, with load/store of
  the retained 8-sample working attachment. Restoration is a preceding pass.
- The ordered suffix is traversed for each disjoint region. Shared glyph drawing
  still issues the prepared instance ranges for each requested text layer;
  a raster scissor does not itself eliminate vertex processing or draw dispatch.

Both are inside the existing suffix timestamp. The result cannot distinguish
their costs or prove that either is the dominant cause. It does not establish
a hardware limit, Wayland defect, or another faulty legacy integration. The
next architectural decision must resolve this attribution before selecting
another renderer correction; rerunning this candidate or tuning parameters
would not answer it. No additional implementation or diagnostic execution is
proposed as implicitly authorized by this report.

Compared descriptively with the previous candidate's p99 of 16.539917 ms, this
trial has no cold frames and a p99 below 8 ms. That is useful progress, but it
is not a matched speedup claim: the first-failure rule prevented the paired
baseline and quiet trials. The p95 failure remains controlling.

## Disposition

Preserve both the successful bounded correctness/resource proof and this valid
performance failure. The approved packet ends after this report and source
assessment. The other eleven declared trials cannot be run under the stopped
campaign. Another implementation/build/native scope needs a concrete proposal
and explicit owner approval; no new scope is ratified here.

S4, S5 and the GPU issue remain open. Camera/resource/observer overhead,
endurance and independent qualification are still required. Complete DRM
lifetime accounting and GPU duty <=25% remain a separate obligation; these
results do not close it. CPU optimization remains deferred.
