# EDA foundation research for S5A derived authority

Status: substantive research dossier, **not ratified product behavior, specification
readiness, implementation proof or engine execution authority**.
Issue: `dat-s5a-derived-authority-hhsw`; step: `DA-A1-RESEARCH`.
Owning route: `foundation-consumer-completion`.
Repository baseline: `ebacde1c640448cff19836dbf99acd7d54641c7e`.
Research retrieval: 2026-10-01. Hypothesis under review: A1-02 at
`c1929bc94a8c408e9e09196b3096b56c3ff97ced`, blob
`8c6025e15cb93dad4cdfd0f214a051fda0deb5a1`.

## Purpose and evidence limits

S5A needs reliable identity and complete membership, but that does not determine
how a professional EDA system should define connectivity, hierarchy, authored
geometry or revision continuity. This investigation starts with concrete design
workflows and inherited Datum authority, then tests A1's proposals against
primary EDA/CAD evidence. It covers the five unresolved domains below, not an
exhaustive industry survey. Evidence was read; no build, native run, fixture,
third-party dependency or implementation was produced. Existing tests and
historical reviews are not new proof. The A1-02 readiness PASS was withdrawn;
this dossier does not restore it.

### Existing authority and observed implementation

| Source | What is established; what is not |
| --- | --- |
| PM000D, “Resolved Electrical Graph Or Cache”; ENGINE_SPEC §0.5 | Stable NetId/anchor versus derived graph; binary anchor-side split, lowest-ID merge, recorded reanchor. Does not fully define simultaneous split/merge over one atomic batch. |
| Formal SCHEMATIC_CONNECTIVITY_SPEC §§4–5; rationale §4 | Formation/scoping rules already exist. Member-derived and name-derived UUID clauses conflict with stable anchor identity; they are not permission to abandon the higher identity contract. |
| CANONICAL_IR §§1–3 | Authored UUIDs, integer-nm input, existing angle representation, derived connectivity. Integer source coordinates alone do not prove exact circular predicates/constructions. |
| PM026; UVT §§2.2.5, 2.2.13, 2.2.18–20 | Progressive section/Run/Net ladder, semantic Bus distinct from Run/scalar Nets, identity lifetime, D4, IR-01, no board Bus projection. Cross-net short traversal and Zone subregion continuity require clarification. |
| NATIVE_FORMAT_SPEC §5.2.1 | Copper derives from current generated fill evidence, not its authored boundary. Successful empty copper needs an explicit representation. |
| `connectivity/mod.rs`; `schematic/mod.rs` | NodeKey is sheet/point; aggregate summaries do not retain complete wire/junction/label identity or instance-path authority. This is observed API insufficiency, not proof that every engine facility is absent. |
| `board/board_types.rs`; `board/net_graph.rs`; `drc/checks/mod.rs` | Track is straight; graph uses net filtering and authored Zone polygon anchors; some distances round floating-point calculations. These facilities do not establish the proposed curved/current-fill exactness contract. |
| `substrate/zone_fill.rs` | Filled currently requires nonempty islands; fully obstructed solver output is Unsupported. Stale revision is distinguishable from a current solved result. |
| `export/copper.rs`; `export/formatting.rs` | Straight track drawing and integer-nm output formatting exist. Complete filled-zone/CAM and curved-copper correctness require call-chain audit, not inference from a function name. |

## Primary reference evidence

These sources inform design; none ratifies Datum policy. Dynamic documentation
and development branches are identified as such; links are not immutable source
archives. A future replay that depends on exact external implementation bytes
must pin those bytes. No code was copied or adopted.

| ID | Source/version | Narrow supported observation and limitation |
| --- | --- | --- |
| P1 | [LibrePCB combine-net command](https://raw.githubusercontent.com/LibrePCB/LibrePCB/master/libs/librepcb/editor/project/cmd/cmdcombinenetsignals.cpp), development master retrieved above | A grouped undo command reassigns schematic/board segments, planes and component signals to a selected survivor with rollback. Supports coherent cross-domain updates; does not justify Datum's ID winner algorithm. |
| P2 | [LibrePCB Bus](https://raw.githubusercontent.com/LibrePCB/LibrePCB/master/libs/librepcb/core/project/circuit/bus.h) and [schematic Bus segment](https://raw.githubusercontent.com/LibrePCB/LibrePCB/master/libs/librepcb/core/project/schematic/items/si_bussegment.h), development master | Separate semantic Bus and drawing-segment UUIDs/references. Supports separating referents; not a released lifecycle guarantee. |
| P3 | [KiCad schematic instance path](https://dev-docs.kicad.org/en/file-formats/sexpr-schematic/#instance-path), developer documentation | UUID paths distinguish repeated sheet occurrences. Supports occurrence context, not automatic adoption of its complete schema. |
| P4 | [KiCad 9 schematic buses](https://docs.kicad.org/9.0/en/eeschema/eeschema.html#buses), 9.0.9/revision 152cd19e | Scalar groups and vector buses, labels and entries have different roles. Does not expand Datum's supported bus syntax. |
| P5 | [KiCad 9 PCB manual](https://docs.kicad.org/9.0/en/pcbnew/pcbnew.html), 9.0.9/revision 152cd19e; Selection, DRC and Net ties | Same-net selection, foreign-net contact checking and designated net ties are distinct concerns. A useful precedent, not proof of Datum's selection graph. |
| P6 | [Open CASCADE Boolean history](https://occt3d.com/dev/doc/overview/html/specification__boolean_operations.html), History Information | Modified/generated/deleted history illustrates operation-derived correspondence. It supplies neither an EDA Zone algorithm nor dependency approval. |
| P7 | [KiCad Track Arc format](https://dev-docs.kicad.org/en/file-formats/sexpr-pcb/#track-arc) | Start/on-arc/end is an existing storage approach; storage alone does not guarantee numerical robustness. |
| P8 | [CGAL circular kernel](https://doc.cgal.org/latest/Circular_kernel_2/index.html), displayed 6.2.1; [robust predicates research](https://www.cs.cmu.edu/~quake/robust.html) | Circular geometry distinguishes predicates and constructions; adaptive robust predicates illustrate filtered evaluation. Orientation/incircle work is not a complete thick-copper kernel. Method references only, no dependency authorization. |
| P9 | [Gerber specification 2026.05](https://www.ucamco.com/files/downloads/file_en/554/gerber-layer-format-specification-revision-2026-05_en.pdf), §§4.7.2, 4.14 | Circular output has finite I/J precision and short/nearly-full arc concerns. Format validity is not a universal internal tolerance or a Datum manufacturing-error allowance. |
| P10 | [Altium circuit connectivity](https://www.altium.com/documentation/altium-designer/schematic/creating-circuit-connectivity), current documentation | Scope can depend on configured hierarchy mode. There is no basis for assuming one universal industry label-scope rule; Datum's declared native/import scope governs. |

## Findings, alternatives and recommended behavior

### R1 — Final-state Net transitions and lawful cross-domain binding

**Inherited:** stable identity, explicit transaction history, no read-side ID
allocation, binary split/merge and coherent component/pin mapping. P1 supports
cross-domain transactional treatment, not a particular identity policy.

**New recommendation:** compare one immutable pre-batch graph with the complete
final graph. Each old NetId nominates one surviving final group through its
surviving anchor; on anchor loss, a canonical surviving old-member occurrence
nominates a replacement. Multiple claims use inherited lowest-ID survival;
unclaimed groups receive recorded new IDs; retired IDs and all binding changes
are recorded in the same batch. Intermediate topology must not allocate IDs.
Anchor-loss fallback and simultaneous transitions require owner ratification.

The alternative is sequential per-operation transitions. It is simpler locally,
but observable IDs can depend on transient order inside one atomic edit. Another
alternative refuses anchor-loss ambiguity for explicit intent; this avoids an
automatic continuation policy but interrupts ordinary deletes. Recommend the
final-state rule with disclosed deterministic fallback and whole-batch refusal
for malformed or unresolved final authority.

Keep logical NetId and board-local Net UUID separate behind explicit relationships.
A schematic split does not magically split already routed copper. Represent
pending/mismatched correspondence without merging logical Nets or falsely
claiming complete cross-pane identity. Valid local inspection remains possible;
per-subject failure cannot masquerade as successful empty membership. Strictly
refusing every temporarily inconsistent board/schematic edit would prevent normal
ECO development and is not recommended. No broad ECO UI is needed to represent
honest incomplete correspondence.

### R2 — Semantic Bus and occurrence identity

A physical spine is a drawing; a semantic Bus is a signal-group/interface subject.
P2/P3 support separating the semantic record, its drawing occurrences and stable
instance paths. P4/P10 show why names/scope rules require explicit interpretation.

Recommend a declared semantic Bus identity with explicit spine/label/entry and
hierarchical interface bindings. Renaming, member order and splitting a drawn
spine do not automatically split its semantic identity. Explicit semantic group
split/rebinding does. Scalar member Nets remain related, not selected geometry.
A full identity-preserving parent/child interface can share one Bus; a subset or
remapped interface relates distinct group subjects unless explicitly declared
identity-preserving. This relationship distinction may amend PM026 and needs
owner ratification, not inference from overlapping scalar members.

The rejected alternative copies Net topology/anchor lifecycle onto Bus: graphic
cuts would then change semantic identity, collapsing the intended Run/Bus
separation. Name/member-set hashes likewise fail rename/member-edit stability.
Occurrence context must enter resolution before unioning local nodes; adding
paths to already-collapsed summaries cannot repair repeated-instance authority.

### R3 — Physical contact, selection Run and shorts

Recommend a Run within the origin's resolved Net assignment, using actual
physical contact on the relevant layers. Independently retain cross-net contacts
for short diagnostics. This is a proposed PM026/UVT clarification: an unrestricted
A+B shorted Run followed by Global Net A would shrink rather than widen the
selection ladder. P5 provides precedent for the distinction, not controlling law.

An unrestricted conductive-region subject is useful for fault tracing but changes
the ladder contract; do not introduce it silently. Intentional net ties preserve
distinct Nets while permitting narrowly declared contact. Current research has
not located complete native tie authority; it must remain an explicit capability
case rather than an invented exemption. This investigation does not authorize a
net-tie feature or a DRC rewrite.

### R4 — Zone component acquisition and continuity

One authored Zone may contribute several current copper components. Source
membership counts it once; the selected Run paints only its acquired component.
A current hit can identify that region; a nonpoint request with several candidates
must ask for a region without selecting an arbitrary polygon. No query fills a Zone.

Replace A1's positive-area overlap identity heuristic. **Recommended new law:**
use validated transaction/generator correspondence when available, exact unchanged
region equivalence otherwise, and disclose reacquisition when neither proves a
unique successor. Splits clear; a certified unique merged successor can survive.
P6 supports provenance as a method, not this exact product policy. This is a
bounded derived-state seam, not a universal topological naming framework.

Overlap is simpler and preserves some refills, but can select unrelated later
copper at the same location and loses disjoint moves. A fixed pointer coordinate
has the same identity defect. The conservative proposal clears more often when
old producers provide no lineage; disclose that cost rather than claiming full
continuity. Stale/missing/unsupported geometry is unknown, not proved deletion.

The current fill schema cannot express successful empty copper. Recommend
allowing **current Filled with zero islands**, with successful producer/basis
provenance, distinguished from Unsupported. This changes generated-evidence
schema/validation and consumers; it is not merely a new selection branch. Never
reinterpret historical Unsupported records as empty or claim an incapable solver
successfully evaluated the geometry.

### R5 — Arc source, numerical truth and manufacturing output

Recommend integer-nm start/on-arc/end Track source (P7), preserving Track UUID,
Net, width and layer. Existing quantized graphic angles cannot generally preserve
arbitrary authored endpoints. Reversal, mirroring, major/minor sweeps and odd-nm
widths must be explicit; integer storage alone does not make derived centers,
radii, offsets or intersections exact.

Recommend certified predicates over source geometry, with fast filters and exact
fallback for supported pairs. Certified inner/outer bounds are an alternative
only when they prove the classification; unresolved cases must not become guessed
contact. P8 establishes relevant methods, not an approved library or implemented
kernel. Nominal tangency, positive gaps and clearance-rule violations are distinct.
Screen tessellation and arbitrary epsilons cannot govern membership or DRC.

CAM has a separate approximation/representability boundary (P9). Three integer
points can imply a center not representable on the output I/J grid. Format-valid
rounding is not automatically acceptable manufacturing error. Recommend exact
representable output first, with explicit refusal when equivalent output cannot
be established; any broader approximation allowance requires its own evidence-
derived manufacturing policy. This conservative choice limits export capability
and must be explicit in the execution decision, not disguised as general arc
support. Unstable arcs may be segmented only under the same declared policy.

Do not invent numeric coordinate limits or error budgets. A kernel proposal must
name its supported domain and checked arithmetic proof before implementation can
claim that scope. Unsupported relevant consumers refuse honestly; an unsupported
router does not disable unrelated inspection. Existing straight output remains.

## Workflow and independent-oracle matrix

These are required future expected outcomes, not executed tests. Source IDs and
producer recipes must be bound before proof; do not use production resolution as
its own expected-output oracle.

| Case | Proposed/inherited expected result and decision enabled |
| --- | --- |
| W01 Display rename / topology label edit | Pure display edit preserves NetId/no allocation. Editing electrical label scope may split actual connectivity. Distinguishes naming from formation. |
| W02 Crossed batch | Before A={a1,a2}, B={b1,b2}, anchors a1/b1; final {a1,b2}, {a2,b1}: A and B survive respectively. Equivalent final batches do not create temporary IDs. Tests simultaneous transition policy. |
| W03 Loss, merge, undo | Anchor deletion uses declared fallback; all old members deleted retires ID; complete merge keeps minimum claimant; undo/replay/reopen restores exact recorded IDs and bindings. No new query allocation. |
| W04 Schematic split / board bridge | Two logical Nets remain distinct while board correspondence reports mismatch; no copper duplication into falsely complete common identities. Valid unrelated subjects still resolve. |
| W05 Repeated sheets | Source at paths I/K has distinct local occurrences; declared globals join; rename/page reorder preserves instance identity; copy creates a new instance. |
| W06 Semantic Bus | Rename/reorder/cut spine: same group, distinct Runs as applicable. Two groups sharing D1 remain distinct. Full interface equality differs from subset/remapping. |
| W07 Short and layer | Same-Net touching joins Run; disconnected same-Net remains only in Net; foreign-Net contact stays outside Run and produces contact evidence. Same XY on disconnected layers does not join. No unimplemented tie exemption. |
| W08 Zone acquisition | Two copper regions of one Zone acquire separate Runs with source count one. Adjacent decomposition slabs electrically join. Keyboard ambiguity refuses; Object and explicit Net remain available where their authority is valid. |
| W09 Zone revision | Equivalent decomposition preserves; certified translation follows; split/unproven successor clears with notice; missing/stale basis suspends rather than proves deletion; current solved empty clears. Undo does not auto-resurrect a dissolved selection. |
| W10 Arc locus | (0,0)–(5,5)–(10,0) mm contacts its bulge but not an isolated chord-only branch. Reverse/mirror, major/minor, tangent/separated/concentric/coincident and odd-width cases have certified or explicit unsupported outcomes. |
| W11 Geometry consumers | Membership, clearance and output use the same authored curve; renderer flattening cannot change classification. Finite-grid CAM distinguishes exact output from unsupported approximation. Unsupported operations cannot emit a chord. |
| W12 Full-set/read boundary | At least 257 members, hidden/filter cases and repeated instances keep complete authoritative resolution. Stale/cancelled/inconsistent subject results never publish partial complete membership; queries leave authored state/journal unchanged. |

## Compatibility, minimum foundation and integration handoff

Implementing existing stable identity and complete query exposure is different
from approving new lifecycle rules. Explicit adoption must preserve native/source
UUIDs and ImportMap provenance; missing identity records cannot trigger lazy query
writes. Instance-qualified resolution cannot be replaced with authored geometry
clones. Version successful-empty fill handling and validate old consumers; unknown
old output is not empty. Arc storage touches typed writes, persistence, predicates,
projection, DRC and manufacturing. These are real prerequisite costs, not renderer
optimizations, and require a capability audit before execution.

Integrate the proposals into the existing A1 coordination packet as exact proposed
owning-clause changes, not an additional competing specification. Preserve PM026
read-only scope, existing native proof, PM049/052 rendering/resize and PM051
undecided performance budgets. The owning homes are PM000D/ENGINE/CANONICAL_IR,
formal/rationale connectivity, native format, PCB/schematic tool contracts and
PM026/UVT. Product changes remain unratified pending complete independent review
and owner disposition. A claim of implementation feasibility or complete supported
geometry requires later proof; this report supplies neither.
