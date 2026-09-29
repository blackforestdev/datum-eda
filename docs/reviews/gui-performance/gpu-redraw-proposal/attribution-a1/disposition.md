# A1 disposition: stopped at source preflight

The owner approved A1. Its execution preflight found an incompatible pair of
requirements in the packet, before implementation or GPU entry. The study ends
**inconclusive, with no measured attribution result**. This is a defect in the
proposed diagnostic method, not a newly demonstrated renderer or driver defect.
The packet should have resolved this before it was presented for approval.

## Demonstrated incompatibility

The approved packet requires both:

> Use one visible native X11 surface and the production format/usage/acquisition
> contracts.

and:

> Validate exact target bytes after every observation, outside timing.

`crates/gui-app/src/gui_runtime_support.rs:200` configures native textures with
`RENDER_ATTACHMENT | (caps.usages & COPY_DST)`. It never requests `COPY_SRC`, even
if the adapter advertises it. `render/frame_target.rs` exposes that same actual
acquired texture as a copy destination; cloning its handle does not change its
creation usages.

Installed wgpu-core28.0.1 `src/command/transfer.rs:1219` checks `COPY_SRC` on the
source of `copy_texture_to_buffer` and returns `MissingTextureUsage` otherwise.
At line1387 it applies the same check to a texture-to-texture copy. Therefore a
staging texture cannot bypass the missing usage. This is a deterministic source
prerequisite failure; no deliberate invalid GPU submission is needed to prove it.
No surface capability failure was observed, and support for a differently
configured readback surface was not tested.

The existing visual capture helper does not solve this: `capture_resource.rs:38`
creates a separate texture with `RENDER_ATTACHMENT | COPY_SRC | COPY_DST`.
`visual_capture.rs:233` reads that capture texture. Reusing it as the diagnostic
target would replace the required acquired native destination. Reading retained
C would verify the source of the last presentation copy, not its destination.
A compositor screenshot would introduce a different synchronization/capture
method and would not establish exact GPU target bytes per observation under A1.

Adding diagnostic `COPY_SRC` to the surface could be a future method choice,
subject to capability and perturbation analysis, but changes the fixed production
usage condition. Waiving destination readback also changes the promised proof.
Neither was silently selected. No successor experiment or renderer change is
proposed or executed here.

## Execution receipt

- Approval recorded against packet SHA256
  `808f9220527ce2f53a0695f70d09fe0d89be0e1763f3ffbcf1112381c8b083c4`.
- Same actual session retained ownership through synchronized Frontier/beads.
- Source and input inspection only; CLI, reference PNG and board file pins match.
- Diagnostic implementation edits: zero. Optimized builds: zero. Cargo
  check/Clippy invocations: zero. GPU processes, submissions and observations: zero.
- No D/P upload or command equality proof, conformance, correctness, timing,
  runtime resource-cap check or marker-effect measurement was performed.
- The stop is at method readiness. Unspent quotas do not authorize a replacement
  packet or an automatic second attempt.

## Consolidated GPU completion assessment

| Question | Evidence and disposition |
|---|---|
| Renderer correctness defect | The demonstrated R4 native attachment-capacity defect was corrected. The preserved R4 workload completed with exact endpoint output, visible crosshair and complete input evidence. A1 identifies no new renderer defect. This is not exhaustive correctness or endurance qualification. |
| Measured budget failure | The valid R4 native result remains p95 6.700167ms and p99 10.205084ms against 4/8ms. The prior corrected painter/hover candidate's p95 4.330583ms remains a separate failed result under its own method, not a matched ranking or proof that another 0.33ms is economical. |
| Backend versus Datum attribution | Unresolved. A1 collected no observations, so it provides no measured backend-operation residual, marker overhead, preparation opportunity or painter/restoration contrast. Historical intervals cannot supply those missing controls. |
| Qualification gaps | Complete DRM lifetime/duty measurement, broader consumer/backend/scale/recovery/endurance coverage and required independent replay remain separate obligations. S4/S5 and the main GPU issue remain open. |
| Feasibility and spending decision | Neither a universal 4ms hardware limit nor an economical path to 4ms is established. A1 adds no evidence justifying another renderer redesign or optimization campaign. Stop speculative optimization; preserve R4 and the prior evidence. |

The approved attribution study is ended, not successfully measured. The overall
GPU repair remains incomplete. This report does not request another approval,
advance to another proposal, or turn a measurement-method failure into a hardware
feasibility conclusion.
