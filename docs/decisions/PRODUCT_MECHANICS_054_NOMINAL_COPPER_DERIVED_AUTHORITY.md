# Product Mechanics 054: Nominal copper and derived selection authority

Status: owner-ratified C04–C06; C07/M1 deferred.
Frontier: S5A-DERIVED-AUTHORITY; issue: dat-s5a-derived-authority-hhsw.

The owner explicitly ratified C04–C06 of reviewed S5A-DERIVED-A1-03 at
`3730cf16f4ca44f37c85bbd011e73bd9ac6e347d`, blob
`7e712f81156da247fa09ffc2e3a2b86ee554c2d4`. These clauses reconcile their
owning specifications and supersede conflicting lower summaries. PM053 C01–C03,
PM026 read-only selection and PM049/051/052 remain controlling.

## C04 — Contact, Run and electrical faults (Q4)

**Owners:** PM026/UVT §2.2.5; canonical IR copper-island definition; PCB connectivity
and checking contracts. Ratified clarification:

> A Run is the complete physically connected component within its origin's
> resolved Net assignment, on actual conductive layers and current geometry.
> Disconnected same-Net members belong to Net, not Run. Foreign-Net physical
> contacts do not merge semantic IDs or expand this Run; they remain separate
> contact/short evidence naming both sides. Net-tie intent can qualify permitted
> contact only through actual authored tie authority, never a name heuristic.
> If assignment or required geometry is unknown, return typed unavailable, not
> an invented Net, assumed gap or short-free result.

This keeps section → Run → Net widening. Unrestricted fault-region selection is
an alternative new subject/gesture and is not added. Current native tie support
must be inventoried before claiming any tie case; unsupported tie fixtures remain
unverified. No broad DRC rewrite or implicit exception is authorized.

## C05 — Current fill and Zone-region selection (Q4 Z1/Z2)

**Owners:** NATIVE_FORMAT_SPEC §5.2.1; PM026/UVT origin/lifetime/projection;
Rendering Book Run versus Object presentation. Ratified clauses:

> Keep one authored Zone UUID. A Run may use an engine-resolved revision-bound
> component qualifier referencing its source Zone occurrence and current copper
> basis. Count the source once while projecting only that Run's connected copper.
> Initial hit acquisition resolves the actual occupied region, not a renderer
> polygon index. Nonpoint acquisition succeeds only with one eligible component;
> otherwise preserve prior selection and explain that a copper region is needed.
> Boundary ambiguity never chooses the first polygon. No selection query fills
> a Zone or treats its authored outline as copper.
>
> Region evolution is engine-owned and old/new-basis-bound: zero, one or multiple
> successor physical components, or Unknown. Accept exact unchanged occupied-
> region equivalence (including decomposition/winding/order changes), or explicit
> transaction/generator lineage validated against current copper. Source UUID
> survival, overlap, proximity and old hit coordinates alone are not lineage.
> A unique successor preserves and rederives; one-to-many split or verified deletion
> clears with explanation. A many-to-one merge can preserve when the mapping is
> uniquely certified. Unknown after current successful fill clears and requests
> reacquisition. Missing/Unfilled/Stale/Unsupported basis suspends the current
> region claim and preserves only explicitly stale prior state; it does not prove
> disappearance. A certified current empty result clears. Undo does not resurrect
> a previously dissolved selection.
>
> Version generated-fill validation to allow successful current Filled with zero
> islands and producer success/source-basis provenance. Such a record asserts
> zero copper. Missing, Unfilled, Stale and Unsupported assert no successful current
> copper result. Never relabel old Unsupported as empty. Old nonempty Filled records
> retain their existing meaning; new readers validate version/basis and new writers
> use the reviewed version contract. Incompatible consumers refuse explicitly.

Recommend exact-equivalence fallback plus optional certified operation lineage,
not a universal topological naming engine. A generator lacking lineage may return
Unknown; the loss of automatic continuity on unproved refills is an explicit
product tradeoff. Final successors are physical components, not polygon count:
several same-Zone fragments joined externally can be one successor component.
Actual lineage validation is mandatory before a producer claims it; no new
producer need invent correspondence to satisfy selection.

## C06 — Arc source and certified nominal geometry (Q4 T1)

**Owners:** NATIVE_FORMAT_SPEC §6.5 Track schema; CANONICAL_IR §§2–3;
PCB tool contract Track attributes; engine connectivity/checking geometry.
Ratified clauses:

> Track optionally stores authored on-arc midpoint beside start/end in integer
> nanometers. Absent midpoint preserves existing straight encoding. Three distinct
> noncollinear points specify the directed circular locus from start through
> midpoint to end, including major sweeps. Preserve UUID/Net/layer/width. Full
> circles require multiple valid sections; do not silently replace them with a
> degenerate triple. Native create/set, schema, inverse/replay and reopen preserve
> these anchors; straight-to-arc edits are Track attribute changes, not new objects.
>
> Nominal occupied copper is the directed centerline swept by round-cap radius
> width/2, including exact half-nanometer radius for odd widths. Checked certified
> predicates distinguish contact (including tangency), positive gap and invalid
> geometry. Rational center/squared radius or equivalent certified representation
> must not round the authored locus to fit the graphic-angle type. Screen flattening
> never determines connectivity/qualification/DRC. Mirror reverses orientation;
> reversal preserves locus with reversed direction. Non-grid transforms disclose
> authored quantization through the owning operation, never through a query.
>
> Supported contact pairs include arc/line, arc/arc, arc/pad, arc/via-span and
> arc/current-filled polygon with holes. Use certified filtering plus exact fallback
> or certified bounds with explicit unresolved result. Boundary cases must be
> supported to claim the required pair, not hidden behind universal refusal.
> Arithmetic domain/range and predicate guarantees must be documented and proved
> from implementation limits; no invented epsilon, numeric performance limit or
> unsafe i64 intermediate overflow. Invalid source refuses before mutation;
> downstream capability failure is source-bound and cannot become chord geometry.

This is real geometry infrastructure. No dependency is approved. Reuse existing
helpers only after their guarantees are established. Exact source storage alone
is not a proof of predicates or constructions; existing rounded distances do not
satisfy this contract by assertion. The minimum implementation may deliberately
refuse relevant routing/fill operations until supported, while valid inspection,
connectivity, DRC and required output remain separately evaluated.


## Bounded execution and deferred manufacturing

The owner authorizes E2–E4 and focused verification of Net-constrained Runs,
separate cross-net contact evidence, Zone acquisition/succession/current-empty
fill, authored Track arcs and certified nominal geometry, complete occurrence-
qualified membership queries, shared selection adapters/projections and nominal
DRC. Reserve the existing independent audit session for exact candidate review
and focused replay. No unapproved dependency or renderer/resize redesign.

C07/M1 is explicitly DEFERRED: no manufacturing approximation allowance is
ratified. Mandatory rational-center CAM/T04 remains pending, not waived or passed.
Defer policy-dependent manufacturing implementation. Bounded nominal execution
may proceed without declaring the complete foundation accepted; complete F01–F06,
independent foundation review/owner acceptance and S5A resumption remain gated.
