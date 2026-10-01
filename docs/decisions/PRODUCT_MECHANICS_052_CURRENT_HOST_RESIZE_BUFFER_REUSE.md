# Product Mechanics 052: Current-host resize buffer reuse

Status: ratified owner decision. Exact response: `GUI-RESIZE-REPAIR: approve PM052 opt-in reuse`.
Recorded in `docs/reviews/gui-performance/cpu-reconciliation/owner-direction.json`.
Tracking: GUI-RESIZE-REPAIR / dat-gui-vertical-resize-cpu-toj.

## Concrete adoption proposal

<!-- REQ:GUI-RESIZE-REPAIR:RZ-C04 -->
<!-- OWNER:GUI-RESIZE-REPAIR:RZ-C04:RZ-C04 -->

Provide an ordinary `--reuse-resize-buffers` launch option selecting the tested
retained-peak allocation policy. The owner can use this option in their normal
release/direct-board launch on the current reference host. Preserve the existing
`quantized` and `quantized-retained` diagnostics and their successful executables.
The original opt-in disposition is superseded by the owner default-path amendment
below. No production preference UI is introduced.

The selected policy grows presentation storage in32physical-pixel increments
only on growing axes, retains peak storage during one active resize sequence,
and restores exact storage after the existing three-current-output-refresh-interval
quiet rule. Logical dimensions, input and design state update immediately. Full
8x rendering, painter order and resource lifetime ownership remain required.
While storage and window dimensions differ, Wayland viewporter presentation
resamples the rendered image. Exact settled pixels remain required. This ratified
narrow transient-output exception does not claim pixel-exact intermediate frames.

Adoption is evidenced only on the current Debian/KDE Wayland, Vulkan/i915 P630,
Mesa25.0.7, fractional-scale1.05 reference configuration and DOA2526 workload.
An explicit option is configuration, not a declaration that other hosts/backends
are qualified. Implementation must fall back safely to exact allocation where
Wayland/fractional-scale/output-refresh prerequisites are unavailable, rather
than panic or guess an output. Scale/output changes must cancel stale peak state
and refresh the output association before reuse. No dependency or protocol client
is introduced.

## Evidence and limitations

The successful quantized observation reduced CPU from38–39% to26–27% of one
core under the recorded geometry workload; the retained variant further reduced
it to21.84% horizontal and19.70% vertical, with fewer configurations. Completion
counts differ, so these are observations under equivalent geometry, not an
identical delivered-work causal partition. The retained variant also reduced
app GPU engine activity compared with quantized. Exact settled captures and
one post-resize quiet-idle interval pass. Owner manual QA positively supports
the original quantized mode; it does not separately qualify retained mode.
Full measurements, failed attempts and boundaries are preserved in
[the repair record](../reviews/gui-performance/cpu-reconciliation/resize-repair.md).

No accepted CPU percentage, hardware ceiling, observer-overhead attribution,
continuous display/input qualification, broad hardware policy or S4/S5 acceptance
follows. Remaining rendering CPU is not proven avoidable merely because it exists.

## Decision requested

Approve the ordinary opt-in retained-buffer feature and its stated temporary
resampling during resize on this reference configuration, while preserving exact
settled output and both successful candidates. This is adoption of the measured
win, not rejection of it or permission to start another performance campaign.
Alternatively specify a correction to this concrete integration scope; all
existing wins remain available regardless of the response.

## Bounded implementation after approval

<!-- REQ:GUI-RESIZE-REPAIR:RZ-C05 -->

Add the ordinary launch option through existing argument parsing and shared
surface services. Preserve diagnostic selections; reject an ambiguous simultaneous
ordinary option/diagnostic override with a clear error. Cover selection, unsupported
prerequisite fallback and output/scale reset with focused tests. Run a guarded
optimized build and affected checks. One visible current-host launch through the
ordinary option verifies full-board readiness and horizontal/vertical grow/restore
with exact settled captures and clean exit. No additional CPU sampling is needed
unless integration changes the tested allocation algorithm or accounting basis.
Update the repair record, Frontier and tracker with the supported result. Broader
resize qualification and all numerical budgets remain separate and undecided.

<!-- EVIDENCE:GUI-RESIZE-REPAIR:OPT-IN-APPROVAL -->

Owner approved the proposal verbatim; RZ-C05 integration is authorized within
the stated current-host scope. No default activation or broad acceptance follows.

## Owner amendment: default workspace rendering path

<!-- REQ:GUI-RESIZE-REPAIR:RZ-C06 -->
<!-- EVIDENCE:GUI-RESIZE-REPAIR:DEFAULT-APPROVAL -->

Exact owner direction: `idealy this is not initiated with a special launch code.
this should be the default rendering path`. Recorded in owner-direction.json.
This authorizes default retained-buffer reuse in ordinary main native workspace
launches, through the same shared surface/render services. No special launch
option is needed. Preserve the previous explicit option for launch compatibility
and both diagnostics; an explicit diagnostic override remains authoritative.
Unsupported Wayland/fractional-scale/output-refresh prerequisites retain exact
allocation. Temporary resize resampling and exact settled output have the same
ratified boundary. Default activation is a policy choice, not broader hardware
qualification; evidence remains current-host/backend/workload scoped. Owned
product-dialog integration and broad UI qualification are unchanged.

Verification earns its cost: test default selection and explicit overrides, reuse
existing fallback/state-transition tests, guarded release build and affected
Clippy. One no-option, no-diagnostic ordinary launch verifies visible readiness,
horizontal/vertical growth/restoration and exact settled pixels, using the existing
controller without resource intervals. No new CPU campaign or budget.
