# S5A derived-selection foundation-gap amendment

Status: proposed planning amendment; independent review and owner approval pending.
Packet: `S5A-FOUNDATION-GAP-01`. Issue: `dat-s5a-derived-authority-hhsw`.
Investigation baseline: `a4537277`; owning route: `foundation-consumer-completion`.

<!-- REQ:UVT-S5A-BUILD:S5A-G01 -->
<!-- EVIDENCE:UVT-S5A-BUILD:S5A-G01-PACKET -->

## Preserved work and operational boundary

The owner authorizes investigation/planning reconciliation only and pauses
implementation depending on missing derived-selection authority. Preserve
`a4537277` partial U1: typed subjects/identities, resolution interface, lifecycle
reconciliation and extraction of predecessor selection methods. Its 19 focused
tests and workspace Clippy passed. Native consumers still use the predecessor;
U1 is incomplete and U2–U5/native acceptance have not passed. The diagnostic
rename test records existing summary behavior, not target conformance.

Keep C01/C02 complete and retain execution approval and independent specification
PASS for `0ef0efc745c8296b12e5027d7039303018d345a8`,
`S5A-C02-PACKET-IR01`, blob `a5d31274d41fec30458d8d51af825815124eb0d6`.
That packet stays byte-identical. This supplement specifies a proposed foundation
prerequisite, not a replacement selection plan or a new grant to implement it.
C03 is pending resumption with its partial work retained; the temporary canonical
sequence is G01 planning → G02 independent-review/owner-disposition boundary.
The Frontier authorization at G02 describes this new decision only; it does not
withdraw the recorded C02 grant. No engine prerequisite is promoted by this draft.

PM026 and D1–D4/IR-01 remain binding: complete membership, origin-only eligibility,
derived-subject preserve/refuse/no-op, read-only acquisition, board/schematic native
acceptance, deferred editors explicitly unverified, latency undecided under PM051.
PM049 rendering and PM052 retained resize adoption (`a89f94b2`/`1fd5193b`) remain.
C05 independent exact-candidate review/replay and C06 acceptance stay separate.

## Requirement, implementation evidence and existing owners

Owners below are architectural/specification lanes, not assigned people or sessions.
The existing incomplete `NATIVE-CONNECTIVITY-CONTRACT` / `dat-grid-connectivity-authority-5s6`
owns reconciliation of electrical identity versus formation/movement clauses
(`specs/FOUNDATION_FOLLOWUP_PLANS.md`, NCC-C01/C02/REVIEW/C03).

| Missing S5A requirement | Observed implementation and gap classification | Existing authority and bounded owning work |
| --- | --- | --- |
| Complete schematic wire Run, distinct from electrically joined disconnected/global occurrences (UVT 2.2.5/20) | `connectivity/mod.rs` has private union-find keyed by sheet/point and hierarchy/name merging. Its aggregate discards wire/junction/label IDs; `SchematicNetInfo` exposes pin/port IDs plus counts. Missing query exposure **and** a physical-component view before logical merging, not missing authored wire IDs. | Connectivity owner; `specs/SCHEMATIC_CONNECTIVITY_SPEC.md` §§3–5 and PM026. Preserve resolved attachment/topology; retain typed source references and expose the origin's full physical component separately from logical Net membership. |
| Complete board Run through track/via/zone, including conductive members (UVT 2.2.5/16) | Board maps already carry authored IDs and Net references. `route_path_candidate_authored_copper_graph*.rs` offers sorted Track/Via/Zone path machinery, but one minimum-step route between pads is not a full connected component. Zone-aware path code tests polygon containment, not demonstrated complete pad/arc/fill contact semantics. Missing component query; reuse suitability requires bounded exactness review. | Board connectivity/geometry owner; PM026 class matrix, engine/PCB contracts. Expose complete physical component with correct layer/contact rules; reuse existing predicates where sufficient. Do not equate same Net, shortest path, zone outline or screen overlap with physical continuity. |
| Stable logical Global Net identity across revisions | `schematic_net_info` hashes label/name and root coordinates; label-only rename diagnostic changes summary UUID with wire/label IDs unchanged. `DesignModel` lacks target `net_anchors`; no `NetAnchor` implementation found in substrate/connectivity. Missing semantic identity implementation, not merely serialization. | Substrate identity/resolver owner; PM000D “Resolved Electrical Graph Or Cache”, `specs/ENGINE_SPEC.md` §0.2/identity and canonical IR. Those targets separate stable Net identity from derived graph. Reconcile formal connectivity §5 merged-member UUID wording and non-normative rationale §4 name-derived UUID wording before choosing exact anchor lifecycle/persistence mechanics. |
| One Global Net subject with lawful board/schematic membership | `get_netlist` chooses board output when present, otherwise schematic summary; no common Net binding projection found. `ComponentInstance`/pinned `PinPadMap` and board Net UUIDs exist, but component joins do not prove a logical Net join. Missing cross-domain binding semantics/exposure. | ProjectResolver/substrate relationship and connectivity owners; PM000D, engine target §0.2, schematic/PCB contracts and PM026 eight mappings. Resolve electrical intent to board Net through explicit authoritative binding; validate contradictions/missing/stale mappings. Name equality cannot establish identity. Bodies remain merely related. |
| Bus section → physical Run → semantic Bus across hierarchy; complete spine/name/entries and scalar-Net relations | Authored `Bus.uuid`, `BusEntry.bus`/`wire`, sheet definitions/instances and ports exist. `get_buses` returns per-sheet UUID/name/member-name records, not a semantic identity/occurrence binding. Hierarchy unions use child root-sheet coordinates without an occurrence path in `NodeKey`; repeated-instance isolation is not established. Missing semantic Bus/occurrence identity and binding, plus query exposure. | Schematic hierarchy/connectivity + substrate identity owners; formal connectivity §§4.4/4.5, PM026 and UVT 2.2.13/20. Define stable semantic Bus binding to authored occurrences using resolved hierarchy; member scalar Nets are related, not selected Bus geometry. Bus has **no board-side projection**. Hierarchical-sheet selection remains excluded; resolving occurrences does not authorize its editor. |
| Exact complete revision-bound inventory for lifetime, inspection and D4/IR-01 | U1 `SelectionResolution` accepts engine-provided sets but has no authoritative producer. It distinguishes unavailable authority from proven missing identity. Class+UUID alone must not collapse distinct hierarchy occurrences or substitute for projection metadata. Derived airwire/fill primitives are not new authored IDs. | Engine supplies one project/revision/source-bound read snapshot; shared gui-protocol adapter consumes it. Selection/viewport owner retains reducer, focus, visibility, inspection and lifecycle. Complete typed member/occurrence refs, owned versus related geometry and scalar Net relations precede caps/filter/rendering. Any occurrence representation change to partial U1 remains explicit migration work, not a second identity system. |

Search/reading covered engine query surface, schematic types/queries, connectivity
net/hierarchy resolver, board maps/queries, authored-copper graph/zone-aware path,
DesignModel/ComponentInstance/relationship loading and native connectivity/
forward-annotation builders. No claim is made that all engine graph facilities
are absent. Existing ERC/DRC summaries and routing witnesses cannot establish the
complete required subject sets. No fresh native experiment was run here.

## Minimum prerequisite and unresolved mechanism choices

Recommend a **new explicit bounded prerequisite**, proposed key
`S5A-DERIVED-AUTHORITY`, using existing issue `dat-s5a-derived-authority-hhsw`
and the existing owners above. NCC is incomplete, planning-only and broader
(junction formation, movement, quantization, Finish/Escape and visual reconciliation).
Do not redefine its entire outcome as the S5A prerequisite or wait for unrelated
gestures. Reuse its identity reconciliation and link both tasks; do not duplicate
its specification authority. Completed M0–M5 or S0–S4 remain closed at recorded
scope (`PLAN.md` legacy freeze); numbering is not a reason to reopen them.

Proposed finite prerequisite units, serial and **not yet authorized**:

1. **A1 contract reconciliation.** Connectivity/substrate owners resolve the
   exact identity, binding, occurrence and query contract through their existing
   specifications and numbered decisions. Independently review and obtain owner
   ratification/execution disposition before A2. Preserve PM026 behavior; escalate
   any required change to its owning route rather than infer behavior from code.
2. **A2 engine authority.** Implement only required stable semantic identity and
   explicit binding support, complete physical/logical membership and hierarchy
   occurrence resolution. Extend existing resolver/typed commit/replay facilities
   only where the reviewed identity contract requires authored state. No new
   selection-triggered writer, general ECO, topology/gesture rewrite or library
   system. Queries never allocate/persist identities or append a journal entry.
3. **A3 read boundary and focused proof.** Expose one coherent project/revision
   snapshot with full typed member/occurrence refs, origin-to-Run/Net/Bus binding,
   related scalar Nets, and distinct unavailable/invalid/missing results. Bind
   source revisions, validate refs and deterministic ordering; no truncation,
   scene names/prefixes/coordinates or capped context as authority. Extend the
   engine query facade; transport additions only if the actual U1 consumer needs
   them, not a new per-class tool vocabulary. Provide reproducible native-project
   fixture variants and exact engine/adapter assertions, not native GUI acceptance.
4. **A4 independent review and handoff.** Independently replay foundation cases
   on the pinned engine candidate/fixtures, disposition failures, obtain exact
   owner completion and explicitly reconcile S5A resumption. No reviewer
   availability is fabricated; request provision before A2. The existing reserved
   S5A reviewer can review this amendment via owner-mediated return; its C05
   reservation does not automatically reserve an engine implementation reviewer.

A1 must resolve these precisely named questions; approval of this amendment
does **not** itself choose their answers:

- **Net anchor lifecycle and compatibility:** reconcile PM000D/engine stable
  surrogate/anchor target with formal merged-member and rationale name-derived
  UUID clauses. Specify ownership of existing native/imported Net adoption,
  rename, split/merge, anchor deletion, undo/replay/reopen and cache invalidation.
  Prefer the existing stable surrogate target over promoting compatibility
  summary hashes. Identity changes requiring authored records belong to the
  canonical transaction, never lazy read-side creation or selection state.
- **Electrical-to-board binding:** identify the exact source record linking
  schematic Net identity to existing board Net identity using lawful electrical
  intent and ComponentInstance/PinPadMap resolution. Specify no binding,
  conflicting binding, board-only/schematic-only and stale revision outcomes.
  Do not choose same-name matching or silently renumber established board Nets.
- **Semantic Bus and occurrence lifetime:** specify semantic Bus identity source,
  authored-section/entry/name ownership, links across hierarchy, repeated-sheet
  occurrence reference and rename/split/merge/deletion behavior. Prefer explicit
  engine stable identity/bindings; do not invent Bus IDs from names, sorted scalar
  member sets or coordinates. Determine lawful reuse of existing source IDs
  before proposing additional persistence. Existing scalar-member syntax limits
  stay honest; unknown/ambiguous syntax is diagnosed, not silently coerced.

Affected homes are `engine/src/connectivity/` (cohesive physical/logical result
owners), `board/` existing topology predicates, `schematic/` typed occurrences,
`substrate/` identity/resolver and necessary typed operation/replay boundaries,
`api/query_surface.rs`, and the U1 gui-protocol metadata adapter. Candidate new
connectivity selection-query modules should be small owned modules, not growth
of an oversized facade or a forwarding split. Canonical identity, engine/native
format/connectivity specs and affected parity/traceability consumers reconcile
in the same changes. No renderer change is required for this prerequisite.

## Focused acceptance evidence and resumption

These batches enable specific decisions; the fixture/oracle is an explicit
independently authored expected identity/occurrence set, not another call to the
production resolver. Reuse adequate existing tests; run new proof only after
separate execution disposition, through the guarded serial Cargo runner.

| Batch | Exact acceptance oracle and decision enabled |
| --- | --- |
| F01 physical versus logical | Branching and isolated board copper; cross-layer via, pad/arc/contact and available zone-fill variants; schematic wire chain/junction and disconnected same-Net label occurrences; local bus component. Complete Run includes every physically connected member, excludes disconnected logical members and unrelated bodies. Compare full typed sets, not one path. Enables safe Run acquisition. Unsupported required fill/contact cases remain unresolved, not passed. |
| F02 stable Net lifecycle | Pinned native source IDs; rename without connectivity change preserves stable Net ID; approved split/merge/anchor-loss outcomes follow A1. Typed commit → revision → fresh resolution, undo/redo, journal replay and reopen agree. No dropped consumer identity substitution/resurrection. Supersede the U1 summary-rename diagnostic with lawful authority proof; retain diagnostic provenance. Enables revision-bound identity. |
| F03 lawful common Net | One explicitly bound board/schematic Net resolves to identical semantic ID with exact per-kind IDs (track/via/pad/zone, wire/junction/label/port/pin); same names on different Nets do not join. Missing/stale/conflicting mapping discloses failure; no name/pin-number fallback. Parent component/symbol identities stay related. Enables same-identity pane projection. |
| F04 semantic Bus/hierarchy | Section, physically connected Run and semantic Bus stay distinct. Linked parent/child occurrences share Bus identity; unrelated equal-name buses do not. Two instances of one definition retain lawful independent local occurrence bindings; global versus local Net scoping follows reviewed contract. Complete spine/name/entry geometry plus scalar-Net relations, hidden/cross-sheet counts and deletion/rename/reopen oracles; no board Bus projection. Enables Bus acquisition and inspection. |
| F05 completeness/eligibility | Full 256/257+ sets with pre-hidden or class-filtered origin/member/intermediate variants. Engine inventory is invariant under pane/visibility/filter/context cap; eligible origin acquires complete set, ineligible origin does not. S5A later N05/N09 supplies native rendering/accessibility proof; this batch supplies exact authority for it and D4 no-op/refusal decisions. |
| F06 read-only/revision honesty | Queries on missing authority, missing identity, dangling member, stale snapshot/source and repeated/reordered inputs return specified distinct results; complete successful serialization is deterministic. Capture model revision, authored shards and journal before/after repeated query/reopen: zero query-induced mutation/identity allocation. Derived cache work is labeled separately from authored state. Enables trusted adapter integration. |

Reuse the pinned S5A fixture families; add only required identity/binding and
hierarchy variants through native facilities after authorization. Bind candidate
commit, actual source inputs/fixture hashes, binary/toolchain/environment,
operation/revision traces, expected and actual full sets, and reviewer replay.
Engine proof does not pass S5A N01–N17, accessibility, native pointer/keyboard,
marquee/refusal/cancellation or final visual acceptance. D1 deterministic work/
resource duties remain; no new latency budget, broad harness or performance run.

**Exact S5A resumption condition:** A1 reconciled and ratified; required F01–F06
cases pass on the pinned engine authority candidate; independent foundation
review/replay has no unresolved mandatory failure; owner records prerequisite
completion and its exact U1 adapter handoff. The handoff supplies lawful stable
Net/Bus IDs, complete revision-bound member/occurrence refs and typed origin
bindings for every approved board/schematic case. Then explicitly close/promote
the prerequisite state and mirror hard-edge satisfaction in beads/Frontier,
select C03 with execution authorization retained from C02, and establish an own
synchronized claim. Resume U1 producer/consumer migration before U2–U5. If the
handoff changes selection behavior or acceptance, reconcile/review that exact
scope change first; otherwise do not request unchanged S5A execution approval
again. C04 proof, independent C05 and C06 acceptance still follow.

## Roadmap proposal and review disposition

Proposed promotion transaction after independent amendment review and owner
approval: insert `S5A-DERIVED-AUTHORITY` immediately before UVT-S5A-BUILD, promote
`dat-s5a-derived-authority-hhsw` from intake to a bounded scheduled prerequisite,
and add that issue as a hard dependency of S5A in both Frontier and beads.
Its initial selected step is A1 **planning**, with an explicit reviewed
owner-decision step before engine execution; do not grant A2 by approving this
planning amendment. Keep a related link to NCC and its unchanged broader
incomplete obligations. Upon authorized implementation/proof/acceptance, explicit
Frontier handoff resumes S5A. Preserve unrelated ordering/ownership and existing
cross-probe dependency; no completed phase or successor is reopened/authorized.

Current transaction schedules only the owner-requested G01/G02 reconciliation
within S5A. The intake remains open/related, not a ratified engine dependency;
the canonical G02 boundary prevents dependent implementation until disposition.
C03 partial progress is preserved in this record despite pending status. C02
review/authorization and original packet bytes are unchanged.

<!-- REQ:UVT-S5A-BUILD:S5A-G02 -->
<!-- OWNER:UVT-S5A-BUILD:S5A-G02:FOUNDATION-REVIEW -->
Independent review must check this complete amendment against the approved
packet, existing owners/authority, missing-exposure versus semantics distinctions,
A1 choices, A1–A4 scope, F01–F06 and resumption gates. Return reviewer identity,
exact amendment commit/blob, outcome and all mandatory finding dispositions.
No implementer self-review or copied C02 PASS satisfies this new review.

<!-- OWNER:UVT-S5A-BUILD:S5A-G02:FOUNDATION-AMENDMENT -->
After independent review, owner approve/revise/defer the exact amendment and
new-prerequisite placement. Recommendation: approve **planning reconciliation
and prerequisite scheduling only**, retaining all S5A acceptance and execution
approval; require reviewed A1 decisions and separate engine execution disposition
before A2. This is approval of a foundation scope addition, not a request to
reauthorize unchanged S5A. Any remaining mechanism/reviewer decision is explicit.

Route review: complete foundation-consumer source and consumer reviewed before
editing; add this supplement to the same route, without changing product routes
or the immutable preparation packet. Their referenced identity/connectivity
clauses are investigation inputs, not silently ratified amendments. No prototype,
runtime, dependency, fixture, renderer or resize/performance change in this turn.
The baseline full drift battery already has the unrelated three terminal-renderer
marker failures tracked as `dat-terminal-renderer-markers-7uuk`; they are not
fixed, waived or treated as passing by this planning amendment. Non-compiling
governance verification is recorded in the landing commit; no native proof or
fresh full Cargo/drift campaign is claimed.
