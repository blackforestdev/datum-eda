# A1 readback correction

Status: concrete scope amendment for owner approval. The owner's `proceed`
direction authorizes preparation of this correction; it is not recorded as
approval of a previously unseen change to the diagnostic configuration.

This amendment addresses the sole demonstrated A1 preflight failure. It does
not replace the renderer or add another optimization hypothesis. The original
packet remains pinned at SHA256
`808f9220527ce2f53a0695f70d09fe0d89be0e1763f3ffbcf1112381c8b083c4`;
its stopped disposition remains historical. If approved, the amended study
resumes its unused implementation/build/run allowance with the following exact
changes. All other A1 controls, requirements, limits and stop rules apply.

## Concrete correction

In the ignored diagnostic test only, create the native surface configuration
through the existing `gui_runtime_support::surface_configuration` owner. Require
that the chosen adapter's surface capabilities include all three usages:
`RENDER_ATTACHMENT | COPY_DST | COPY_SRC`. Check the pinned format, extent,
backend, adapter identity, scale and Fifo contract as before. Then add
`COPY_SRC` to the diagnostic configuration before its first configure call.
Assert the acquired texture carries all three usages on every acquisition.

The normal configuration helper and normal executable behavior stay unchanged.
Do not add a production option, fallback backend, alternative target, shader or
native event-loop path. Failure to support the complete usage set stops the
single process; no substitute configuration is authorized.

N,D,M,P,A,B,C all use this one configuration, one device/window and the actual
acquired texture. Do not reconfigure between conditions or introduce an eighth
condition to compare surface usages. The R4 final C-to-target copy remains in
each measured graph, including its common timestamp bracket where applicable.

After the measured graph completion, read the acquired texture into the existing
bounded diagnostic image-readback buffer, compare its exact raw format bytes
with the appropriate reference, unmap, and present it. Do not use retained C or
an offscreen substitute as evidence of destination correctness. Record image
readback submission/wait/map/comparison and presentation-call wall durations
separately from the measured graph. Presentation is not a display acknowledgement.
Use the same order for every condition, including N. The query readback behavior
and its separately reported cost retain A1's N/D/M distinctions.

The expected output remains the exact new full-reference image for N/D/M/P and
the identical initialized-prefix hybrid for A/B/C. All observations still check
output; the correction waives neither fidelity nor final-copy inclusion.

## Fixed allowance and implementation boundary

A1 spent no diagnostic build, Cargo check, GPU process or observation. Approval
of this amendment releases only the unused original allowance, not an additional
batch on top of it:

- One optimized offline guarded diagnostic test build; at most two scoped
  guarded check/Clippy invocations. First compiler/check failure stops, no retry.
- One GPU process, one device/window, at most300seconds including setup and
  cleanup; no separate capability probe process.
- Exactly the original two cases, seven conditions,16 conformance observations,
  14 correctness observations,56 warmups and560 measured observations:
  maximum646. Preserve the original ordering and first-failure stop.
- At most2200 total queue submissions,64MiB diagnostic CPU storage,16MiB numeric
  output and512MiB total tracked GPU ownership. Source preflight must account
  for upload/reset, graph, query/image readback and conformance submissions before
  entry. Combine compatible untimed initialization/upload draining as required
  within existing owners; never enlarge the allowance or omit a required drain.
- Existing A1 diagnostic feature/test/helpers only. No new dependencies,
  shaders, observer thread, production optimization, tile sweep or DRM execution.
- D/P upload and ordered command parity, bounded accounting, exact8 and timestamp
  conformance remain prerequisites. This amendment does not claim they passed.

Implementation review must verify the usage addition is confined to the ignored
feature-gated test entry, the acquired destination itself is read, and the graph
completion interval ends before diagnostic image readback. Seal source/binary
pins and run the original offline negative controls before the single GPU entry.
No build is needed to review this usage correction itself.

## Interpretation and overall completion path

Adding `COPY_SRC` can change swapchain allocation or backend transitions even
when no readback lies inside the measured interval. Keeping it identical across
all seven conditions controls the configuration difference between those cells;
it does not prove zero cost or equivalence to the production surface. Per-frame
readback and delayed presentation also condition cache state and sampling cadence.
No production-usage versus diagnostic-usage comparison is authorized, so that
perturbation remains unmeasured and must be explicit in the report.

Apply A1's predeclared block medians, paired contrasts and materiality rules
unchanged, but interpret every outcome as conditional on this diagnostic
configuration. Marker contrasts remain N/D/M comparisons within it. A measured
copy/resolve residual is the cost of the selected graph in this configuration,
not an unavoidable hardware floor. It cannot establish production4/8ms
acceptance, decompose historical native p95, or independently prove production
R4 should be abandoned. Do not subtract an assumed usage/readback overhead.

The study can identify whether painter/restoration/preparation or marker changes
produce a reproducible material contrast within the fixed graph. A clear signal
supports only an explicit value decision about that narrow source of work.
It does not authorize its optimization or another renderer design. If controls
fail or attribution remains inconclusive, the study ends with no successor.

This correction repairs measurement readiness, not a renderer defect. R4's
preserved exact-output native evidence and measured budget failures remain.
Hardware feasibility and the economic value of further optimization remain
unknown pending usable observations, and may remain unknown afterward. Complete
DRM duty/lifetime and broader S4/S5 qualification remain separate obligations;
no GPU issue closure follows from this diagnostic study.

Approval response: `approve GPU attribution A1 readback correction`.
