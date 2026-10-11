# PM061 — Effective Padstack Process Geometry

Status: ratified by the appointed manager under the confirmed exact owner
process-policy delegation; independently reviewed policy, implementation pending.
Roadmap: `EFFECTIVE-PADSTACK-PROCESS`, `dat-effective-padstack-process-vgno`.
Authority: the owner's exact confirmed process-policy delegation, recorded in
`docs/reviews/workflow-delivery-rollout/effective-process-owner-delegation.json`.
This is a new disposition under that delegation, not a historical owner reply.

## Purpose and scope

One engine service resolves native mask and paste apertures. Library inspection,
explicit placed materialization, checking, rendering and manufacturing consume
that result. A raw payload query alone does not deliver this feature. Reads never
select defaults, adopt capabilities, change profiles or rematerialize placements.
PM008, PM057 and PM058 continue to govern source identity and exact transforms.

<!-- REQ:EFFECTIVE-PROCESS:PROCESS-P01 -->
## Selection, inheritance and basis

A Footprint may author a separate typed `process_basis` selector: `inline`,
`ipc_basis`, or `project_profile`. Preserve `process_aperture_policy` as existing
descriptive string metadata; do not parse `ipc_derived` or other strings as a
selector. An absent selector supplies no inheritance authority.

An inline basis contains concrete non-inheriting mask/paste policies, an explicit
paste ratio in ppm and authored provenance. An IPC selector uses the Footprint's
existing structured `ipc_basis` values. Nonnegative mask expansion and paste
reduction produce their declared positive and negative offsets respectively;
reject malformed signs instead of repairing them with absolute value. This
decision supplies no new IPC formula, number, package family or standards default.
The IPC selector supplies an explicit zero paste ratio: its existing structured
basis has a reduction value, not a ratio. Never borrow a profile ratio for it.

A project-profile selector names a stable UUID and expected canonical object
revision and value hash. Profiles live in an optional UUID-keyed
`process_profiles` collection in native ProjectRoot. Each profile records its
UUID, name, concrete mask/paste policies, explicit paste ratio, declared basis
and provenance. Policies cannot inherit again. Author and select them through
typed, guarded canonical operations with journaled preimages and Undo/Redo;
never through preferences, implicit globals or a private file writer. Preserve
absence on historical ProjectRoot reads. Deletion of a referenced profile must
refuse until explicit dependent selections are reconciled.

Explicit Padstack policies override the selected basis for that process. Absent
raw policy and explicit `Inherit` both request the selected Footprint basis,
while inspection retains their distinct raw presence. No missing selector falls
back automatically to a project profile. No values from unrelated sources mix.
Explicit paste `Aperture` or `ExpansionNm` supplies zero ratio; a ratio is used
only with the selected inherited paste basis or preserved authored placed input.

Missing, ambiguous, stale or unsupported required resolution input is unresolved,
with a source-specific reason. Unknown standards evidence is a separate finding:
explicit geometry can be resolved and usable with unknown standards basis.
Checks disclose that uncertainty; manufacturing never labels it certified.
Unresolved geometry prevents the affected output or new materialization.

<!-- REQ:EFFECTIVE-PROCESS:PROCESS-P02 -->
## Policy and physical layer membership

Mask `Exposed` explicitly uses the copper outline and zero offset; `Tented`
suppresses the opening. Paste `Aperture` explicitly uses the copper outline and
zero offset/ratio; `None` suppresses the opening. Signed `ExpansionNm(e)` offsets
each dimension by twice the declared value. Suppression is an explicit resolved
result, distinct from unresolved geometry or legacy missing authority.

BoardRoot declares front and back exterior copper IDs and, independently for
each face, optional mask and paste layer IDs. Validate copper IDs against the
first and last Copper entries of the declared physical stackup sequence, process
IDs against their actual SolderMask/Paste types, and unique process associations.
A single copper layer may serve both exterior faces; the process associations
still distinguish those faces. Never infer sides from names or numeric distance.

Resolve PadLayer, Through and Blind copper spans through the existing placement
and span authority. For each process, intersect the reached copper span with the
declared exterior faces; enabled apertures occur only on those faces. Through
does not create an aperture when its process policy is suppressed, and no span
creates internal mask or paste. An enabled reached face without the corresponding
declared process association is unresolved. A span reaching no exterior face has
an explicit empty membership for that process.

Reflection and placement-side changes use the existing copper placement transform
to exchange front/back associations, including blind-span endpoints. Source-
preserved explicit placed membership remains authoritative: present empty arrays
mean no openings; missing membership is unresolved rather than empty or all layers.
Retained legacy consumer behavior remains identifiable as legacy, not new native
process authority or certified output.

<!-- REQ:EFFECTIVE-PROCESS:PROCESS-P03 -->
## Exact geometry and adapter limits

Native Padstack circles and rectangles retain their shape and centre. Compute
each dimension independently with checked exact integer/rational arithmetic:

```text
mask:  effective_dimension = copper_dimension + 2 * margin_nm
paste: effective_dimension = copper_dimension * (1 + ratio_ppm / 1000000)
                            + 2 * margin_nm
```

The ppm denominator is the unit conversion, not a tolerance. Native explicit
ExpansionNm has zero ratio. Preserve exact rational dimensions and centres,
including half-nanometre boundaries. Reuse existing exact placement transforms;
never round through float, clamp collapsed geometry, or substitute a bounding
rectangle for unsupported retained shapes. Nonpositive dimensions and overflow
produce explicit source-specific errors before any transaction or artifact write.

Rendering and CAM consume canonical effective geometry and membership. Display
conversion is presentation only. An adapter must represent its supported result
exactly or refuse with a specific unsupported-result error before publishing any
partial output set. Positive delivery includes integer-nanometre circle and
quarter-turn rectangle output on both faces, with nonzero offsets and ratios
whose results are exactly representable. Odd dimensions retain their exact centre.
Nonintegral ratio results and other rotations remain exact query data and require
explicit adapter support or refusal. Refusal does not qualify those manufacturing
cases. PM058 rational-centre and rotated CAM obligations remain open separately.

<!-- REQ:EFFECTIVE-PROCESS:PROCESS-P04 -->
## Frozen placements, staleness and reviewed updates

Explicit canonical materialization freezes effective geometry, membership, raw
policy presence/values, chosen basis/profile identity, complete library/profile
and stackup pins, resolver version and transform basis. Placed consumers use that
snapshot, not current library values resolved during a read. Profile and stackup
pins participate in current/stale status as library pins do.

Library, profile or relevant stackup changes expose stale placements, their frozen
result, the proposed current result and exact differences. A process-changing
placed update requires a reviewable prepared proposal. Manual users can prepare,
inspect, reject and apply it through the same deterministic operations as agents.
Editing the library alone never changes placed geometry or approves propagation.

Apply validates model/project revisions, placement preimages, source/profile/
stackup pins and complete PM057 correspondence before staging. Preserve placed
Pad IDs only through validated correspondence, never names, coordinates or guessed
identity. Record allocations, retirements and old/new bindings. Reject incomplete
or stale batches atomically. Undo/Redo/replay restore recorded frozen process data
and identities without consulting newer policy to recompute historical geometry.

<!-- REQ:EFFECTIVE-PROCESS:PROCESS-P05 -->
## Capability adoption and compatibility

Introduce a separately declared BoardRoot process-source capability for typed
face associations and new frozen process snapshots. Adoption is an explicit
guarded journaled transaction with the actual byte preimage; it is not a change
to historical profile-4 `AdoptBoardSourceProfile` semantics. Existing profile and
operation replay retain their historical meanings.

Use a versioned process snapshot nested in the existing binding, omitted when
historically absent. New validation has explicit legacy and process-capable
branches. No read-time defaults, rewrite, implicit adoption or synthetic process
snapshot is permitted. Explicit suppression, resolved geometry and absence of
process authority remain distinguishable. Adoption itself supplies neither
geometry nor permission to apply a placed update. Do not claim that process Undo
reverses forward-only profile adoption. Reject unsupported declared versions.
The new capability adoption is also forward-only: Undo or replay of an inverse
adoption refuses without changing authored state or journal. Replay of its actual
forward entry validates the recorded preimage and restores its exact declared
capability and associations. Ordinary later profile, association and process
materialization edits retain their canonical Undo/Redo. Such edits must preserve
the capability and validate all dependent snapshots; an incompatible edit or
history application refuses atomically rather than downgrading or orphaning them.

Preserve source-resolved imported inputs without reopening import conversion.
No dependency/license/source adoption, HUMAN assessment, GUI visual ratification,
standards certification or fabricated manufacturing threshold follows from PM061.

<!-- REQ:EFFECTIVE-PROCESS:PROCESS-P06 -->
## Complete delivery and acceptance

The finite delivery contract covers native manual authoring and basis selection;
effective inspection; explicit frozen placed materialization; process rendering;
mask/paste export with independently parsed exact geometry comparison; library/
profile edits and stale evidence; reviewed update rejection/application; and
Undo/Redo/replay/reopen with canonical identity and journal integrity.

Positive ordinary proof covers both faces, circles and rectangles, positive and
negative process offsets, an explicit nonzero paste ratio, suppression, membership
and odd dimensions. Meaningful negative controls cover missing inherited basis,
ambiguous roles, stale proposals, incomplete correspondence, unsupported exact
consumer geometry, overflow and collapse without authored/journal/artifact loss.
Retain old-profile and source-preserved compatibility proof and distinct review.
Metadata-only inspection, a safe refusal or an isolated fixture does not replace
required positive rendering and manufacturing behavior. Qualification claims must
name actual supported cases and retained gaps; no full manufacturing PASS follows
from this bounded feature.

## Evidence and implementation ownership

Existing complete foundation and parametric route reviews are reused only at exact
unchanged artifact pins verified against `2c34dac0195688dc686f372dce5de29ed2971a67`.
Current source establishes the missing persistence and consumer seams; it does not
establish standards authority. New policy and consumer changes require distinct
review and deliberate same-change route reconciliation.

Owners are shared engine process resolution, canonical project-profile authoring,
BoardRoot capability adoption, component Pad materialization/persistence/projection,
existing library/proposal CLI workflows, GUI protocol/render adapters and engine
process export. The manager owns policy, task synchronization and actual independent
receipt acceptance. One serial guarded proof batch owns each necessary build.
