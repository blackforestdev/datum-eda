# S5A exact 30° placed-Pad selection proposal

Packet: `S5A-C03-EXACT-30-01`. Status: proposed, independent review and explicit
owner execution authorization pending. Frontier: UVT-S5A-BUILD / S5A-C03;
issue: `dat-uvt-s5a-build-1wv`. Source baseline:
`b2c1421538f201484b16be1e89590a7193cd30c7`; delivery packet blob
`956088a43f72229d5ebc349e7238786c09d1b584`.
Owning evidence route: `foundation-consumer-completion`.

## One decision and delivery boundary

Recommend approving query-only exact source-relative geometry for the mandatory
30° placed-Pad/Footprint path, including the scalar representation, contact
predicates and typed consumers below. After focused independent specification
review, record a numbered product-mechanics amendment (or explicit numbered
PM054/PM057 amendment) and authorize this bounded implementation. A specification
PASS alone does not authorize it. Current approval covers this proposal only. Proposal-only exact coefficient examples
from the unchanged historical eight-Pad source are pinned in
`thirty-degree-proposal-evidence.json`; they are not runtime/contact/native proof.

The delivered behavior must include ordinary-board selection and rendering of
these placements, both-face 4-of-8 refusal / 5-of-8 Footprint acquisition, and
complete source-qualified Pad/Run/Net membership. All eight centres are required;
missing one cannot become a placement-point fallback or a reduced denominator.
Existing parent/child click grammar stays unchanged: normal owned-Pad acquisition
collapses to Footprint; eligible section → Run → Net and existing explicit member
verbs retain their contracts. No new gesture or Pad-origin widening is proposed.

Reuse PM026/053/054/057, preparation N03/N09, current worker/publication contracts
and existing source fixtures. No persistence/profile/adoption, identity, tolerance,
CAM/T04, dependency, prototype, budget or scope change is proposed. No general
angular authoring, native aperture-family expansion or geometry rewrite is granted.

## Representation: source pose, exact coefficients and compatibility

Existing `FrozenPadGeometry` already records integer-nm local position, Circle or
Rect aperture, drill, layer/span and plating. `PlacedPackage` records integer-nm
origin T, degree rotation and copper face. Journal/provenance and pinned library /
ComponentInstance / PinPadMap authority already exist. Read derives from these
records; it never mints IDs, rematerializes, adopts a profile or writes coordinates.

Let M = diag(-1,1) on the back face, identity on the front, and let q be the
recorded local centre. Derive the world point and whole aperture by
`F(p) = T + R30 M p`; its inverse is `M R30^t (p - T)`. Thus, after reflection,

```
Cx = Tx - y/2 + (x/2)√3
Cy = Ty + x/2 + (y/2)√3
R30 = [[√3/2, -1/2], [1/2, √3/2]]
```

The exact centre type is `Quadratic3 { a: Rational, b: Rational }`, denoting
`a + b√3` in nanometres. Rational is the existing normalized checked i128 pair:
positive denominator, reduced fraction, unique zero. Integer/rational injection
sets b=0. Equality compares normalized coefficients, never rendered values.
Multiplication is `(ac+3bd, ad+bc)`; inverse divides `(a,-b)` by `a²-3b²`.
For opposite signs, sign(a+b√3) compares a² with 3b² with the sign of a retained;
same-sign and zero cases need no squaring. There is no numerical sqrt call.

Q(√3) is closed under the linear maps, dot/cross products, squared distances,
clamping, line intersection and rational-radius predicates needed below.
It is **not** closed under contact with existing exact 45° features. For those
mixed pairs only, use the bounded compatibility field
`K = Q(√2)(√3)`: `K { u: Q2, v: Q2 } = u + v√3`, equivalent to four rational
coefficients of 1, √2, √3, √6. Inject Q3 as `(a,b)` with zero √2 terms; inject the
existing Q2 owner as `(u,0)`. Multiply `(u,v)*(s,t)=(us+3vt,ut+vs)`; inverse
uses `(u,-v)/(u²-3v²)` in Q2. Sign reduces to signs of u,v and u²-3v² in Q2.
This private pair adapter preserves the existing 45° owner; it is not replacement
of working integer, quarter-turn or Q2-only predicates. Unsupported unrelated
angles retain current certificates/refusals; this does not waive required cases.

Proposed engine types: `ExactPointNm=[Quadratic3;2]`, `SourceRelativePadGeometry`
(source identity/revisions, frozen local geometry, T, M, exact rotation basis,
centre and layer groups), and `QueryPadGeometry::{Integer(PlacedPad),
SourceRelative(SourceRelativePadGeometry)}`. K is predicate scratch, not source
or Inspector data. No implicit or lossy conversion to `Point`/`PointNm` exists.
An explicit integer extraction succeeds only when b=0 and a is an integral i64.
Read-only construction keys include project, model/source revisions, pinned
materialization and actual pose; equal display centres never establish identity.

## Decisions beyond centre calculation

| Decision | Exact construction/predicate and required result |
| --- | --- |
| Pad point/hit | Transform the integer/rational query into the Pad frame. Rect uses inclusive local half-width/height comparisons; Circle uses squared distance ≤ radius². Subtract the strict interior of the centred drill disk. Screen hits nominate IDs only; authoritative qualification uses this predicate. |
| Footprint anchors | Pass every derived centre to the shared region owner. Region triangulation stays rational. For each triangle, edge cross products with the Q3 point have exact Q3 sign; all three nonnegative signs include boundaries. Union triangles implement the existing even-odd region. Count complete anchors once and require inside > total/2. No area/majority rewrite. |
| Straight Track–Pad | In the Pad frame, transform both endpoints. Reuse finite endpoint, segment projection/clamp, corner and edge-crossing candidates from occupied-copper/eighth_turn. Parameters and squared-distance comparisons stay Q3 (or K for mixed compatibility). Width/2 and odd-nm halves remain rational. Include equality for contact. |
| Circular Track–Pad | Preserve CertifiedArc source start/mid/end, rational centre and squared radius. Transform centre/endpoints/directions into the local field, without converting the rational centre to Point. Reuse arc endpoint, edge foot, radial-gap and circle/edge-intersection candidates and directed major/minor sweep guards. A constructed intersection is represented transiently as A+B√D, A,B,D in the base field, D≥0. Its sign reduces by guarded squaring; it is never rounded into a query point. |
| Pad–Pad / Pad–Via | Native Circle/Rect and existing imported Circle/Rect/Oval/RoundRect have finite line/circular boundary features with field-valued centres/endpoints and rational radii. Enumerate segment/segment, segment/circle and circle/circle intersections, tangencies and coincident-overlap candidates. Check candidates against both occupied sets. Also test one exact representative per boundary component to detect containment without crossings. Include each drill circle as a boundary component, so outer-shape overlap entirely inside a hole cannot join a Run. Different supported poses use K only where needed. |
| Pad–current fill | Use existing certified occupied cells, never Zone outlines. Test transformed Pad/fill boundary intersections and containment using exact edge signs / even-odd cell membership. Hole interiors remain absent cells. Do not omit any island, replace current fill, or make a query fill a Zone. |
| Run / Global Net | Existing complete graph walks and PM053 correspondence stay owners. Each supported Pad is inserted exactly once per certified layer group, preserving plain placed ID and no schematic instance path. Same-Net contact forms Run; disconnected same-Net remains Net; foreign-Net contact remains separate evidence. Missing/stale binding or a required unresolved pair returns typed unavailable, never complete partial membership. |

The finite-boundary containment rule applies to the validated connected apertures
with strictly contained drills already supported. Each boundary is piecewise line /
circle; an occupied-set containment change requires a boundary crossing. With no
crossing, one witness per connected boundary component determines containment.
For a circle/arc, choose a certified on-sweep endpoint or cardinal representative;
never an approximate sample. Coincident boundary intervals require certified
interval overlap, not “first intersection.” Each candidate's membership uses
exact field signs and the same directed arc guards. This is a proposed predicate
construction, not a claim that current RoundedBox/certificates already implement it.

The drill guard for Track contact must also move to the same field. Endpoints
alone do not certify an arc inside a hole: include its farthest on-sweep extremum.
For nonnegative distances, a comparison of `√U+√V` with L first checks L and
`W=L²-U-V`, then compares W² to 4UV with the appropriate equality rule.
For `A+B√D`, determine signs first; only opposite signs compare A² with B²D.
These sign reductions need no persisted/general root-valued point or trig kernel.
Negative radicand, zero denominator, ambiguous/coincident case without an exact
handler or checked range failure is a typed error, not guessed separation.
Required boundary cases cannot pass by this refusal; implementation must prove them.
A zero drill contributes no hole boundary; zero-length segments become points.
Coincident circles/segments and discriminant-zero tangencies need exact branches,
so degeneracy is not silently treated as separation.

## Affected typed consumers and integer compatibility

| Owning source today | Bounded change after authorization |
| --- | --- |
| engine `substrate/component_pad_source/{projection,validation}.rs` | Retain provenance/materialization/layer checks; construct a complete typed query-Pad map alongside unchanged canonical source. Validate local aperture without creating a fake integer world centre. Share immutable terminal metadata for electrical binding. |
| engine `substrate/{electrical_basis,electrical_correspondence}.rs` | Current `project_board_value` inserts PlacedPad JSON into an ephemeral Board Value and correspondence deserializes it. Replace that dependency for exact placements with a typed query-geometry/terminal sidecar owned by the same captured basis. Source class/history registration and package/net/PinPadMap checks remain exact; no new source shard or persisted geometry map. |
| engine `substrate/electrical_selection_query{,.rs}`, `connectivity/board_physical{,/bounds}.rs` | Capture the complete typed Pad map in the same snapshot; Shape gains a source-relative branch. Exact point/contact dispatch uses the shared owner above. Bounds exclude only certified disjoint pairs; inability to certify bounds must retain the pair, never discard it. Existing membership/source-basis DTOs do not change. |
| engine `board/occupied_copper{,/eighth_turn,/quadratic,/certified_polygon}.rs`, nominal geometry | Add bounded Q3 and mixed-pair adapters and source-relative predicates in normal modules; reuse Q2/CertifiedArc and existing pair algorithms. Model-bound nominal DRC uses the same predicates where it consumes these Pads. No broader DRC/routing/fill rewrite. |
| protocol `placed_pad_projection.rs`, native source extraction in `lib.rs` | Stop serializing exact Pads through integer BoardPadPayload. Native extraction consumes typed query Pad geometry. Integer/imported payloads stay lossless and retain existing source semantics. |
| protocol `PadPrimitive`, `board_review_scene.rs`, `selection_scene{,/index}.rs` | Explicit native scene version2 carries `PadCenter::{Integer(PointNm), Exact(ExactPointNm)}` plus source-relative aperture pose. Version1 scenes decode losslessly as Integer. Update actual consumers together; a reader not supporting2 must refuse, never ignore exact Pads or substitute zero/local centres. Scene bounds and stable source keys remain derived/revision-qualified. |
| renderer `render/{pad_retained,pads_and_layers,native_selection_overlay,scene_relationships}.rs` | Evaluate the same exact pose to floating screen coordinates only at painting. No integer-nm tessellation/centre is claimed as authority. Exact pose, source ID and layer group qualify selection bindings; cached geometry keys include pose/revision. Existing integer painting/ownership stays intact. |
| protocol `selection_geometry_catalog.rs`, viewport `selection_region.rs`, app `selection_region_evaluation.rs` | Add ExactAnchors using the existing shared majority rule; retain rational region triangulation and resumable atomic evaluation. Certify conservative integer AABBs by exact floor/ceil comparisons; those bounds only enumerate candidates. Never use them as the anchor set. |
| protocol `selection_view/{properties,inspection,quantities}.rs`, context/AT-SPI | Expose derived board-space position as typed exact Q3 quantities, separately qualified from unchanged local source coordinates. Compare coefficients for Common/Mixed. Transport reduced numerator/denominator as decimal strings with radicand3 and unit nm, avoiding JSON numeric loss. Read-only symbolic display converts coefficients to declared Project units exactly; absent units remain Unavailable. No rounded display value can feed geometry. |
| CLI `commands/project_core.rs`, Gerber `copper_context.rs`, legacy Board/routing/fill consumers | Existing integer Board consumers use explicit exact-integer extraction only. If unsupported, preserve existing source-bound refusal before output/artifact replacement. GUI extraction and selection use the typed query path and therefore are not disabled by that export limitation. No new manufacturing positive, approximation or CAM/T04 waiver. |

Scene version2 and exact-position transport are the proposed transient protocol
contract changes requiring disposition. They do not change native BoardRoot4,
canonical Point, operation payloads, journal inverses or source serialization.
No fake Point is permitted even as an adapter placeholder. Existing integer
consumers either remain on their adequate path or explicitly report unsupported.

## Arithmetic domain and refusal guarantees

No new coordinate or latency budget is proposed. Existing normalized Rational
coefficients have signed i128 numerators and positive i128 denominators; every
sum/product/division checks intermediate range and cross-cancels as today.
For initial30° centre coefficients, widening i64 inputs before reflection gives
numerators `2Tx-y`, `x`, `2Ty+x`, `y`, denominators2. Their magnitudes are at most
3·2^63 and 2^63, strictly below2^65; thus even i64::MIN reflection needs no i64
negation. Subsequent predicate intermediates do not inherit this small bound.

Pure Q3 final sign comparison cross-multiplies four unsigned≤128-bit factors
and a factor3: <514 bits, fitting the existing nine-u64-limb pattern (576 bits).
Factor3 must be proved explicitly; the existing factor2 implementation is not
that proof. This is bounded private sign scratch, not a bigint dependency.
Q2 retains its existing final sign comparison. Mixed K and nested-root reductions
require field products first; any reduced coefficient/intermediate outside i128
returns ArithmeticRange. No claim covers all possible i64 Track triples or all
possible contact intermediates. The admitted domain is inputs for which the
required checked constructions/predicates complete, as in PM054.

For query AABBs, bracket √3 between1 and2 with coefficient sign respected, then
use exact coefficient comparisons to find floor/ceil; no floating bounds may
exclude candidates. Bounds outside the existing world/index range produce its
explicit source-bound capability failure or an unpruned engine candidate, not
clipping. Required fixture operations must be shown admitted; a range refusal
on the mandatory30° case remains a delivery failure, not acceptance evidence.

Current frozen-native aperture authority supports Circle/Rect only; preserve
that limit. Imported Oval/RoundRect, rational Track arcs and Q2 poses keep their
existing validators; the new pair adapter supports required combinations without
promoting other arbitrary angles. Unknown layer/barrel, stale/unadopted binding,
invalid/noncontained drill, invalid/current-fill absence, malformed polygons and
arithmetic/certification failure preserve existing explicit outcomes. No required
case is deferred by listing these limits.

## Finite implementation / proof requested after approval

One coherent feature unit: engine exact query Pads and all listed selection /
scene consumers, then ordinary native input. Reuse the existing eight-Pad library /
PM057 placed-Pad and PM053 electrical binding fixtures and the recorded0/30°
parent recipe. Historical unadopted parents are evidence, not lawful electrical
inputs: prepare only the necessary qualified variant using existing canonical
materialization, pose/flip and relationship operations after execution approval.
Do not edit frozen fixtures in place or expand unrelated matrices.

Prove these delivery questions with independently enumerated IDs/geometry:

1. Front and back30°: all8 exact centres/poses; four contained anchors fail and
   five acquire only Footprint, with the existing focus grammar. Rectangle and
   lasso boundaries use exact membership; on-edge, translated and reflected
   cases discriminate rounded-centre substitution.
2. Native Circle/Rect contacts with straight and directed circular Track, Pad,
   Via and current filled cells: positive, tangent, gap and drill-exclusion
   witnesses. A30°/existing45° pair exercises mixed-field compatibility; reuse
   unchanged historical45° proof for the original owner.
3. Complete physical Run and semantic Net sets with authoritative registration /
   binding, disconnected same-Net, foreign-Net contact and layer/barrel behavior.
   Compare complete source IDs, not counts. Unsupported dependent authority is
   whole-result unavailable; it never produces a partial advertised complete set.
4. Ordinary launch: visible rotated placement, region majority, section Run/Net,
   Escape and subsequent interaction. Existing asynchronous resolution,
   revision qualification, cancellation and atomic publication must survive
   source/pose changes; no stale result, fake ID or partial geometry is published.
   X11 evidence retains its configuration; native Wayland/HUMAN proof is explicit.
5. Selection, inspection and reopen leave every source/journal byte unchanged.
   Existing canonical rotate/flip undo/replay/reopen retains recorded source
   bytes and rederives the same pose; no graphics/profile adoption is implied.
   Legacy exact integer consumers and export refusals preserve prior artifacts.

Record demonstrated failures and repair their cause before claiming delivery.
Focused arithmetic/pair/source tests justify native correctness; they do not
replace ordinary input. No unrelated performance campaign or proof matrix.

## Reserved independent specification review return

Return this exact commit/blob to the existing owner-facing audit/review session
through the owner-mediated route reserved in preparation D5. No new reviewer,
independence, PASS or C05 implementation replay is inferred. Review must disposition
representation closure including mixed45°, finite contact/drill/sweep completeness,
checked domain/admission, typed consumer/version compatibility and the preserved
acceptance criteria. Report mandatory findings individually and pin any corrected
packet before owner authorization. No reviewer transport endpoint is recorded in
this repository; if none is supplied, the pinned return is delivered to the owner
for that reserved session, with actual transmission explicitly pending.

Exact requested owner response after review:
`S5A-C03-EXACT-30-01: approve the reviewed query-only Q3 representation, bounded
Q2(√3) mixed-contact adapter, source-relative predicates and typed scene-v2 /
exact-position consumers; authorize the complete mandatory30° behavior and
focused ordinary-build proof; preserve all stated source, identity, cancellation,
nonmutation, manufacturing and acceptance boundaries.`
