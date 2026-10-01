# Product Mechanics 053: Electrical identity authority

Status: owner-ratified C01–C03 only, 2026-10-01.
Frontier: S5A-DERIVED-AUTHORITY; issue: dat-s5a-derived-authority-hhsw.

## Exact authority and execution boundary

The owner ratified C01–C03 of S5A-DERIVED-A1-03 at commit
`3730cf16f4ca44f37c85bbd011e73bd9ac6e347d`, blob
`7e712f81156da247fa09ffc2e3a2b86ee554c2d4`. Independent review already
recorded in `2a294c16` is reused. The following clauses are the ratified rules,
not approval of C04–C07. C01–C03 resolve conflicting lower-level name/member
UUID and topology-based Bus lifecycle wording. PM000D authored identity and
canonical mutation authority remain; PM026 read-only selection is unchanged.

## C01 — Stable Net identity and atomic transitions (Q1)

**Owners:** PM000D “Resolved Electrical Graph Or Cache”; ENGINE_SPEC §0.5;
formal SCHEMATIC_CONNECTIVITY_SPEC §5; rationale connectivity §4.1;
CANONICAL_IR §§1/3. Preserve authored UUID and one-transaction laws. Replace
name/member-derived semantic UUID wording with this ratified clause:

> Connectivity groups are derived from source at one model revision. Stable NetId
> is authored identity resolved through its anchor and recorded transition history;
> a name, union root or member digest is not a semantic NetId. Connectivity edits
> plan identity once from the immutable pre-batch groups to the complete final
> groups. Intermediate operation states allocate no semantic identity.
>
> Each old NetId nominates at most one final group. A surviving anchor nominates
> its unique final group. If the anchor is deleted, choose the minimum surviving
> old-member occurrence reference and nominate its final group; record replacement
> anchor and reason. The canonical reference ordering is class tag, source UUID
> bytes, then ordered instance-UUID path bytes, independent of names/coordinates.
> No surviving old member means no nomination. Malformed duplicate final membership
> or a multiply resolving anchor is invalid authority and refuses before writes.
>
> A final group with nominees retains the lowest nominated old NetId. A group with
> no nominee gets exactly one new UUID recorded in the transaction; final groups
> are ordered by sorted complete occurrence references for allocation recording,
> never hashed into identity. Every other old ID retires with transition provenance.
> An old ID cannot be retained by two groups or recycled by a second-pass heuristic.
> Record final anchors, retirements, allocations and affected bindings in the same
> canonical authored batch. Read/cache rebuilding, undo/replay and reopen allocate
> no IDs; replay uses the recorded source history.

Automatic anchor-loss fallback and final-state nomination are **owner-ratified product
clarifications**. Alternative explicit transition-intent refusal would avoid an
automatic choice but interrupt ordinary deletions. Operation-sequential identity
would expose transient order inside an atomic batch. Recommend the stated rule
because it extends existing automatic reanchor and lowest-ID law without hidden
query writes. Separate commits may legitimately have different histories.

Do not promise IDs remain stable when every original member was deleted and new
unrelated source appeared at identical coordinates. Pure display rename preserves
identity; an electrical label edit that changes grouping invokes the transition.
Correct the schematic tool contract's omitted rename-label explanation accordingly,
without adding a new GUI tool or changing inherited label-formation/scoping rules.

## C02 — Binding and incomplete design states (Q2)

**Owners:** ENGINE_SPEC §0.5 relationship/source model, native-format source
ownership/schema, schematic/PCB tool contracts' connectivity synchronization.
Proposed clause:

> Logical NetId, board-local Net UUID and relationship ID are distinct typed
> references when unequal. Relationships bind them using explicit electrical
> intent and ComponentInstance, occurrence and pinned PinPadMap evidence; equal
> names or reference strings never create identity. BoardOnly/SchematicOnly intent
> is explicit and differs from pending, stale or contradictory correspondence.
>
> A schematic edit may leave a valid authored pending/mismatched board relation;
> it must not silently rewrite board copper or duplicate its ownership to claim
> both new logical Nets. Such a Net cannot yield a complete cross-domain Global
> Net selection. Return a source-bound unavailable/inconsistent subject result;
> valid local Object/Run diagnostics and unrelated complete subjects may remain
> available. Never publish a partial Global Net as complete or treat a failed
> binding as successful empty membership. Subset relation is not identity equality.

Q2 introduces no general ECO editor. Incomplete design state is lawful; dishonest
complete selection is not. Atomic transaction validation still refuses malformed
references and unauthorized writes. Explicit adoption of old projects preserves
source UUIDs and imports only proved identity provenance; no startup/query repair.

## C03 — Semantic Bus and occurrence scope (Q3)

**Owners:** formal connectivity §§4.4–4.5; ENGINE_SPEC resolved-model contract;
native-format hierarchy/identity records; PM026 Bus projection and UVT §2.2.13/20.
Proposed clause:

> A semantic Bus is an authored signal-group/interface identity, not a physically
> connected spine or hash of scalar members. Bound spine/label/entry occurrences
> project that identity; scalar Nets remain related. Cutting or redrawing a spine
> changes its Run, not the declared Bus identity. Rename and member ordering retain
> BusId. Adding/removing scalar membership without an explicit semantic split also
> retains BusId; identity is the declared group, not its current member set.
> An explicit semantic split names exactly one resulting group retaining the old
> BusId and records a new UUID for each other resulting group. The command supplies
> the complete member/binding distribution; every prior member reference and
> representation binding is accounted for, with any intentional removal or new
> shared membership explicit rather than inferred. A representation occurrence
> has one Bus owner; duplicates or missing rebindings refuse the entire batch.
> Explicit merge names one existing surviving BusId, records retired group IDs,
> and rebinds all affected representations in the authored batch. No automatic
> geometric split/merge law allocates Bus IDs. Removing the final drawing leaves
> the semantic declaration resolvable with zero drawn projections; explicit
> inspection can still address it. Explicit declaration deletion or recorded
> retirement during semantic merge ends that identity, after removing/rebinding
> dependent relationships in the same batch.
>
> An occurrence is source reference plus ordered stable SheetInstance UUID path.
> Resolve local topology in that context before unions. Display names/page order
> do not alter occurrence identity; copying creates a new instance identity.
> Native label scope follows the declared native connectivity rules. Unsupported
> imported bus syntax is diagnosed rather than silently promoted to native law.
>
> Full declared identity-preserving interfaces may bind occurrences to one Bus.
> Subset/remapped interfaces retain distinct Bus subjects linked by explicit
> scalar-role mappings; shared names or scalar members never prove equality.
> Such related-interface projections do not receive same-identity selection cues.
> A Bus has no board projection.

This changes A1-02's topology-based Bus lifecycle and extends the relation contract;
the owner ratified it here; PM026/UVT are reconciled in the same change. Existing
scalar syntax limits remain; arbitrary bus-language expansion is out of scope.

## Bounded E1 execution and independent review

Owner authorizes E1 implementation/verification only: stable Net identity and
final-state transitions, explicit logical-to-board pending/mismatch relationships,
semantic Bus declarations/occurrence/interface bindings, schema/adoption and
canonical mutation/inverse/undo/replay/reopen. Routine representation choices
belong to existing engine/substrate owners. No selection-triggered writer,
scene-generated identities, broad connectivity rewrite or unapproved dependency.

The existing owner-facing independent audit session is reserved for the exact
E1 candidate and relevant identity, binding, Bus and persistence replay. This
reservation does not replace complete foundation review or S5A C05. Partial E1
success cannot close this prerequisite or resume dependent S5A implementation.
M1 is unresolved: retain complete manufacturing obligations without invented
tolerance or narrowed CAM acceptance. C04–C07/E2–E4 remain unratified/unauthorized.
