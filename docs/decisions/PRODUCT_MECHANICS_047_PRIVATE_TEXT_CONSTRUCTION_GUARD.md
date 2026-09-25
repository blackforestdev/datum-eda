# Product Mechanics 047: Monitored private text construction

Status: ratified by the owner on 2026-09-24.
Issue and Frontier: `dat-gui-performance-implementation-vkq`, GPI-S4.

## Owner decision

The owner replied `I approve your recomendation` to approval of the exact
private text construction amendment proposed in commit `fdd16f33`, recorded at
`docs/reviews/gui-performance/implementation/S4/private-allocation-guard-proposal.json`.
The recommendation explicitly described transient overshoot and lack of an
out-of-memory recovery guarantee. All six proposed clauses are applied together.
This changes the acceptance mechanism; it is not a sequencing-only amendment.

## Mechanism and limits

Existing private font selection/loading/cache and glyph raster construction may
use the shared monitored call guard specified by amended MEM-02/ACC-02 instead
of hard pre-call admission. Observe full instantaneous and per-call peak owned
bytes, allocation overhead, and simultaneous host/process incidence. Before
publication or GPU submission, an overrun must reject preparation, release
newly derived work as appropriate, and preserve authoritative text and pending
damage for explicit retry. Before/after snapshots and lifetime high-water alone
cannot establish per-call peaks. Retained inputs/outputs and temporary scratch
must be distinguished without double counting shared allocations.

A private call can temporarily exceed its budget before rejection. This is not
protection against process-level allocation failure. Any observed overrun fails
qualification; detecting it does not make it compliant. Required content cannot
be silently omitted. All numerical caps remain unchanged, including 16 MiB
host and 64 MiB aggregate staging/scratch. Datum-controlled allocations retain
pre-allocation admission. Full accounting is still required.

## Delivery and authority

GPI-S4 must implement the default font/raster guard and refusal/release/retry
paths with affected positive and negative verification before S5. All other
carried S4 implementation requirements remain. GPI-S5 retains complete in-scope
qualification, including cold load, churn, simultaneous hosts and recovery.
Approval satisfies no implementation predicate and closes no milestone.

No new, modified or vendored dependency is authorized. PM046 remains historical;
PM029 owner-only authority and existing registry dependencies are unchanged.
Preserve GEFN, visual behavior, source authority and nonblocking resize exclusions.
