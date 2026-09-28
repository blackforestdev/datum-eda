# Product Mechanics 049: Shared render-session ownership

Status: ratified by explicit owner approval of pinned GPU shared-renderer proposal r2.
Issue: `dat-gui-performance-implementation-vkq`; Frontier: GPI-S4.

## Owner authority

The owner replied `approve GPU shared-renderer proposal r2`. The exact response,
proposal hash and commit are recorded in
`docs/reviews/gui-performance/gpu-redraw-proposal/owner-approval-r2.json`.
The controlling implementation and proof scope is the immutable
`proposal-r2.md` with SHA256
`c66c53938cbc9bc708b0132884f5d37eec5570e93f34acb21e74b6c8baafb784`.
Independent review remains advisory; this owner response supplies ratification.
This decision supersedes PM048's active integration and proof boundaries.

## Shared ownership

Implement one shared renderer-owned per-host RenderSession for derived content,
prepared composition, typed accumulated changes, immutable frame plans and matching
completion receipts. Share existing charged document payloads across hosts/panes.
Editors retain authoritative model/session/input/camera intent and submit versioned
inputs; they do not mutate render cache validity or select passes. Preserve the
existing accounted history mechanism and caps while moving its rendering authority.

Retire the proposal's duplicated nullable prepared-state/preparation paths, direct
prepared camera/grid mutation, editor-facing unversioned composition bypass and
raw single-board/companion encoder branches after normalized consumer adoption.
Preserve genuine empty compositions. One prefix/suffix encoder owns painter order
for full and incremental strategies. Main, dialogs and capture share the contract;
capture cannot publish native hits/damage success and dialogs allocate no prefix.

NativeFrameCoordinator retains native damage/token scheduling and fairness;
SurfaceTransaction retains configure/acquire/present; QueueOwner retains submission
admission/completion. Bridge their identities/attempts with session receipts; do not
replace platform policy or optimize the CPU event loop. Terminal snapshots/damage
retain rollback/merge leases through deferred and failed preparation/presentation.

## GPU strategy and resources

Retain one optional exact8x prefix A, ending immediately before schematic
interaction. A warm eligible interaction-only plan copies all samples to equal
format/extent/sample B, draws the unchanged ordered suffix once and resolves once.
No repeated prefix/world/grid draw or unchanged world upload. Cold frames store
prefix A then copy; full fallback shares the same painter owners. Stronger content,
composition, target or mixed changes dominate interaction hints. Unknown inputs,
failed/foreign/old receipts and recovery cannot validate stale images.

Use tracked texture/view aliases counted once; A and B counted separately under
all existing caps. Optional A <=45MiB/host; image-key metadata <=4KiB; actual-size
allocation. One current and one retiring coherent pair-generation, aggregate GPU
cap512MiB unchanged; retain submitted allocations until actual completion and keep
allowances across replacement. Optional retention must not starve required content.

## Approved proof scope and remaining qualification

Implement in the r2 ordered units and run its contract/adoption/exact-output,
invalidation/failure/resource-lifetime controls, including deliberately faulty
controls. Source-health extraction and mandatory repository checks remain required.
No tolerance or golden refresh, dependency/license addition or prototype edit.

After correctness and copy-inclusive timestamp/input/output conformance, r2 permits
one pinned candidate's descriptive W-POINTER comparison: three baseline/candidate
pairs in each quiet/GPU mode (12runs maximum), fixed5s warmup/30s schedule plus
identical idle/drain, alternating AB/BA, first failure/invalid stop and no replacement
runs. W-PAN one pair per mode (four additional runs maximum) checks affected cost.
The exact r2 thresholds, calibration, resource/CPU checks and stop rules control.

Complete DRM accounting is not falsely inferred from timestamps or endpoint scans.
While incomplete, the bounded experiment is explicitly nonqualifying and duty is
unavailable. It cannot close S4 or establish hardware impossibility. Add workload
fd-lifecycle/drain/reset method design to the repair; select/review exact mechanism,
privileges and overhead before a separate concrete observer execution scope. This
ratifies no unknown ptrace/tracing mechanism or global/system-tuning campaign.

S4 still requires approved implementation with exact8x correctness and required
performance, including GPU p95<=4ms/p99<=8ms and complete duty<=25%, affected caps and
all original obligations. S5 broader qualification/independent replay/owner product
acceptance remains separate. Preserve all225requirements, historical failures,
PM045/PM047/PM029 authority, nonblocking exclusions and CPU optimization deferral.
