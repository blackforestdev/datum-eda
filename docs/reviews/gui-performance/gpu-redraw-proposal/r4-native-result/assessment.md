# R4 corrected native result and completion assessment

R4 is preserved. The demonstrated native attachment-ledger crash is corrected
in `c3949e53`, and R4 has its first valid native pointer result. The result fails
both GPU timing limits. The fresh fixed campaign stopped at C-G1; eleven later
trials did not run. This is a measured budget failure, not another invalid run
or observer-capacity failure. The overall GPU repair remains incomplete.

The correction uses the shared renderer's existing ten-entry current/submitted
image bound in the native ledger, replacing its stale three-entry assumption.
It preserves allocation identity, current/retiring membership and last-use
completion. The 23 native queue/ledger tests pass, including five current plus
five submitted images and close/device-loss retirement. Default/visual all-target
Clippy and repository gates passed. The prior R4 exact8 GPU proof is reused:
the image graph, shaders, painter and rasterization were unchanged. No observer
redesign, renderer replacement, dependency or CPU/DRM work was introduced.

## First valid native evidence

The reference P630/Vulkan/X11 run completed the declared workload with 3,600
requests and all 1,280 changed integer pointer positions received and routed.
All 1,278 GPU frames are accounted for: two startup, 1,275 active and one close.
The input observer retained 2,571 of 4,096 records without overflow. Readiness
and final images each have zero differing pixels from the pinned reference;
the final image was inspected and has the visible crosshair. Final state matches
the last applied pointer. Controlled drain and process exit completed normally,
no processes remained, and the candidate binary hash was unchanged.

This establishes stability for one complete 30-second active pointer workload:
no startup crash, lost input/frame receipts or observed output mismatch. It does
not establish repeated-run stability, long-duration endurance or every native
lifetime/recovery condition. Those obligations remain explicit.

| Complete active GPU span | Result | Requirement |
|---|---:|---:|
| Median | 4.839250 ms | Descriptive |
| p95 | 6.700167 ms | <=4 ms: failed |
| p99 | 10.205084 ms | <=8 ms: failed |
| Maximum | 12.937750 ms | Descriptive |

988 of 1,275 active frames exceeded 4 ms; 32 exceeded 8 ms. All active frames
used upload-leading, restoration, suffix and final-marker passes; there were
zero active world-upload submissions and no cold/full graph. The measured
failure therefore persists on the intended regional warm path. It is not
explained by the earlier non-pad hover cold-invalidation defect resurfacing.

## What the retained timings establish

Restoration-pass p95 was 1.307250 ms; suffix-pass p95 was 3.909000 ms. The
interval from suffix end to final-marker start had p95 2.358833 ms, and the
whole interval from suffix end through final-marker end had p95 2.616833 ms.
The final-marker pass itself had p95 0.021583 ms but p99 4.872500 ms. These are
separate distributions; adding their percentiles would be incorrect. Raw ticks,
complete spans and descriptive distributions are retained in `analysis.json`.

Source shows a restoration draw per selected tile, an ordered suffix traversal
per selected tile, resolved tile copies into C, and a complete C-to-presentation
copy. R4 removed the full-size warm multisample resolve, but the remaining
regional restoration, painter and copy sequence still costs too much under
this method. The after-suffix interval includes copies and associated transitions;
the final marker can also include waits/scheduling. The existing timestamps do
not isolate pure copy cost or explain the long marker tail. No attribution to
hardware limits, Wayland defects, clocks, or a specific driver operation follows.

The prior candidate's valid p95 4.330583 ms and p99 5.701750 ms remain historical
measurements of that candidate. R4's values are numerically higher, but this is
not a matched regression estimate: the method now includes the final marker,
and no baseline or quiet trial ran after the candidate failure. Neither timing
observer overhead nor a diagnostics-off comparison is established. No samples
or marker costs were removed to make the candidate appear compliant.

## Remaining path to completion

The native integration blocker is removed and tracked defect
`dat-r4-native-attachment-capacity-uquy` can close against `c3949e53` and this
valid run. R4 remains the shared rendering foundation; the result does not
justify discarding its correctness or replacing its architecture speculatively.

The immediate unresolved renderer obligation is performance of the regional
warm graph. Existing evidence narrows attention to restoration, repeated painter
work and post-suffix copy/marker intervals. It does not yet establish the precise
limiting operation or a proven correction that meets 4/8 ms. Performance
feasibility remains unknown. An unchanged repeat cannot convert this valid
failure into acceptance; further work must address supported costs while
preserving R4 and its exact output/resource contracts. No new optimization
mechanism or execution campaign is ratified by this assessment.

Once a candidate meets the pointer limits, repeated/matched evidence and the
remaining camera/consumer/backend/scale, resource/observer overhead,
endurance/recovery and independent native qualification still remain. The
owner's separate UX/product disposition is also required. This one valid
workload does not close S4, S5 or the main issue.

Complete DRM lifetime accounting and <=25% engine duty remain a separate
obligation. The unresolved context-accounting finalization barrier is still a
method-feasibility gap; neither successful presentation nor queue timestamps
solve it. That gap does not invalidate this focused pointer result, but blocks
final qualification. CPU event-loop optimization remains deferred.

The repair has moved from a native integration failure to a correct, measured
regional renderer with a demonstrated performance shortfall. The remaining
uncertainties are now performance attribution/feasibility, repeated stability
and broader qualification, plus the independent complete-duty method. There is
no defensible claim of overall completion or a guaranteed final optimization.
