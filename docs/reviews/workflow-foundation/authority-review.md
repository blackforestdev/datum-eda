# Foundation authority review — working evidence

Status: WDQ-F01 in progress; no foundation contract or product acceptance.
Tracking: `FOUNDATION-WORKFLOW-SPEC` / `dat-manual-foundation-contracts-fsw`.
Review baseline: `82623dde` (2026-09-08 UTC).

## Ownership and purpose

The owner directed this session to finish the development-quality rollout and
explicitly identified GP-CM05V as another session's work. Commit `82623dde`
records this separate foundation planning lease. Preferences selection, scope,
evidence and owner acceptance are unchanged. This record preserves review work;
it is not a new specification, enrollment policy or substitute roadmap.

<!-- EVIDENCE:FOUNDATION-WORKFLOW-SPEC:WDQ-F01-PARTIAL -->
## Reviewed baseline and unresolved authority

PM001, PM002 and PM012 were read completely. None is listed as a source or
consumer in the evidence traceability manifest at this baseline. All three are
classified as doctrine by the governance manifest, but their document headers
retain draft language. This is a clause-status ambiguity, not permission to
ratify every open question or discard explicit requirements.

| Source section | Observed requirement or qualification | Consequence for the foundation review |
| --- | --- | --- |
| PM001, Resolved Mechanism | Journal history, durable undo, identity persistence and object/model revisions are explicitly ratified | Preserve these settled mechanisms; do not ask the owner to choose session-only history again |
| PM001, Direct Commit Versus Proposal; Open Owner Questions | Local visible undoable manual edits have direct-commit criteria, but the document retains detailed policy questions and historical migration ordering | Check later decisions before classifying a question as still open; a historical question list alone is not current authority |
| PM002, Decision; First Proof Slice | Native manual tools must use typed operations and the journaled commit path; proof includes binding, electrical/physical edits, checks, output and reopen | Engine capability alone does not establish native delivery; the bounded foundation contract must state its consuming handoffs without claiming this entire proof slice |
| PM012, QG-DIRECT-EDITING-FEEL | Preview is session state; cancellation leaves no committed mutation; fields validate before commit | Future scenarios need separate preview, invalid-input, cancellation and committed-result observations |
| PM012, QG-CRASH-RECOVERY; QG-DURABLE-UNDO | Failure before and after the journal commit point has different expected recovery; undo survives reopen | Unchanged-source reopen from the pilot cannot discharge edit recovery or durable undo |
| PM012, QG-PERFORMANCE-LATENCY | Preview and transaction latency are distinct; numeric budgets depend on representative fixtures and owner decisions | Do not invent thresholds or reuse Preferences budgets as acceptance of board editing |
| PM012, owner correction dated 2026-08-23 | Floating/PiP/multi-monitor composition requirements were withdrawn | Do not revive those requirements while resolving draft-header ambiguity |

These are source observations and review obligations, not a completed
cross-document reconciliation. PM012's header/classification discrepancy remains
unresolved pending later-authority review. No decision header, mechanism, route
digest, golden or acceptance receipt was changed to resolve it prematurely.

## Process route review

The complete ten-file `workflow-delivery-quality` route was read: its five
research sources, quality plan, PM041, gate contract, adoption packet and pilot
JSON. Existing route digest is
`ec4b5a58ecf43f323e5a05ca25f9ede2de3e972716db5a6d83a35d075c843e0b`.
No member or digest was changed. Important preserved limits:

- The pilot intentionally excludes authored geometry, numeric entry, snapping,
  library edits and durable edit undo. Those N/A dispositions cannot be copied
  into an authoring contract.
- The adoption packet calls for measurements on the pilot and three separately
  enrolled slices. Pilot completion is not evidence of that broader adoption.
- Research uses the existing planning/owner-decision mechanism. It does not need
  a fake infrastructure delivery contract to proceed.
- PM041 enrollment expansion and trust promotion require exact owner review;
  the instruction to finish the rollout is not a fabricated acceptance receipt.

## Selection route review and concrete discrepancy

At `89a6da10`, the complete five-file `gui-selection` route was read: both
research reports, both derived guidance documents and all 2,627 lines of the
Universal Viewport Tooling specification. PM023 and PM026 were also read in
full. The unchanged route digest is
`5fe8bbe378e16a050e3dcba2d6b16de20010377752d2feea70f50930bf69c629`.
This establishes source review, not fresh runtime or visual conformance.

The following F3/F4/F5 inputs are already specified and must not be reopened
as arbitrary implementation choices:

- UVT §2.2.4: pad clicks select the parent footprint in the board workspace;
  child pad editing belongs to the Footprint Editor. Region qualification is
  based on authored anchors, not glow, bounding-box appearance or zoom.
- UVT §2.2.18 and PM026: identity-preserving edits preserve selection;
  deleted identities are dropped and reported, never replaced by fuzzy matches;
  undo does not resurrect dropped consumer selection. Pane focus changes do not
  change the shared selected subject.
- UVT §2.2.14 and PM026: S5A is read-only. Later locked/stale/incompatible/
  constrained/invalid batch members refuse the complete operation without a
  journal effect or silent member removal.
- UVT §2.2.12: later movement shares one translation rule and one command
  across invocation surfaces. Its reference is invocation-specific; a selected
  focus member is not an implicit pivot or extra mutation authority.
- UVT §3 and PM023: object snapping overrides grid within the eligible radius;
  exact target coordinates are nanometers, while radius is screen-space.
  Gesture snapping is consumer state; committed coordinates enter typed
  operations. Quantize is an align parameter, not an additional private verb.

**Tracked specification discrepancy: `dat-selection-envelope-run-flr`.**
UVT §2.2.19's closed context `subject_kind` list omits Run, while §2.2.20
explicitly includes Run and requires a one-to-one envelope vocabulary. PM026
ratifies Run as a distinct derived identity. The Proposal/Review versus
`review_action` representation also needs explicit reconciliation. This is a
document-level contradiction; no runtime failure is claimed. Intake is related
to this foundation review and the existing S5A owner, without amending their
hard dependencies or accepting a replacement schema.

The correction must preserve PM026, review both owning routes before editing
the shared UVT specification, and name
exact vocabulary/serialization parity proof. Read-only S5A cannot be made an
editing feature to bypass the discrepancy. The historical matrix's code-status
columns remain historical observations, not a current implementation audit.

## Selection visual-authority review

At `8a743d8f`, the remaining `prototype-selection` members were read completely:
the selection study's HTML/CSS/SVG and the Rendering Book. Together with the
already-read UVT specification, this completes the three-file source review.
The unchanged route digest is
`cf208f77ef12aee380bdb7ce04c433e9352982b31bf712cabdca99d018c9a359`.
No new render, visual approval or native conformance result is asserted.

The study's nine panels cover construction, compound equality, locked objects,
channel collision, pane projection, Global Net/Bus, text/points, dense fallback
and accessibility/zoom. They illustrate visual behavior, not an alternative
wire schema for the selection envelope. The pending-review banner is historical
wording: UVT §2.2.21/22 and PM026 record the later owner approval. It must not be
used to reopen the approved visual choices or to claim a new approval here.

For `dat-selection-envelope-run-flr`, the reviewed sources supply no reason to
alter the prototype. The bounded reconciliation is a specification mapping and
its proof: carry Run without losing origin or revision-derived membership;
make Proposal/Review representation explicit; preserve same-identity versus
merely-related mappings; and test every supported wire round-trip without
silently collapsing distinct subjects. Any actual change to PM026's identity
law requires owner disposition, not a cosmetic digest refresh.

Future exact-edit proof must separately demonstrate that selected geometry
retains material/semantic appearance, pane focus alone identifies GUI mutation
authority, selection never changes hit geometry or authored buffers, and
aggregate rendering never truncates membership. A screenshot of a selected
object cannot prove correct target identity, mutation, cancellation or undo.

## Units, field entry, and recovery handoff inputs

At `7615ca98`, the shared Units requirement, GP-C04 storage/recovery contract,
GP-C05 interaction contract, and V1 descriptor catalog were read completely.
These are four further members of the existing settings route, not a completed
route reconciliation or an audit of the other session's implementation.

The following distinctions narrow what the later foundation contract must
connect and what evidence cannot substitute for it:

| Reviewed source | Existing requirement | Foundation handoff consequence |
| --- | --- | --- |
| Shared Units, Lossless display and edit contract | Focus regenerates exact suffixed text from canonical geometry; unchanged Enter/blur is a no-op; Escape restores the pre-edit value; only a changed canonical value journals | F2/F5 need observations of canonical value, dirty state and journal before/after each action, not merely a parser test or screenshot |
| Shared Units, exact grammar and adapters | Scalar decimal/scientific input uses checked rational conversion; sub-nanometer input refuses; explicit suffix is profile-independent; bare input requires declared field/Project context | Authoring consumes the existing service and context, not a new field parser. `5.08mm`, `200mil`, `0.2in` and `5080000nm` supply the existing cross-surface equivalence oracle |
| Shared Units, surface and seed ownership | Project Reset reads the immutable seed/migration receipt, never current Global defaults; missing receipt makes Reset unavailable with a typed reason | F1/F8 distinguish Global default changes, Project display-setting mutations and authored geometry changes; a settings-only no-geometry proof cannot establish exact board editing |
| GP-C04 §§4–5 and 10 | Preferences commit point is repository head promotion; corrupt machine data is preserved, recovery is offered, and authoring remains available; machine recovery never mutates Project authority | F6 must separately specify Project journal recovery and writer ownership. Preferences backup/restore proof cannot discharge Project edit/reopen or durable undo |
| GP-C05 §§4–9 and catalog §2 | Reserved controls stay absent; active controls need real consumers, truthful provenance, keyboard access, non-color state and accessible announcements | Do not activate reserved grid, autosave or library preferences to fill an authoring gap. Existing editor behavior and future preference activation are separate obligations |
| GP-C05 §7 and Shared Units lossless editing | Refused input remains available for correction without changing the last valid value; cancellation and successful commit are distinct | Future numeric-entry scenarios need invalid/intermediate entry and recovery, not only accepted final strings. Field-level transient grammar and focus choreography still require the complete authoring review |

The reviewed Units requirement explicitly rejects persisting authored expression
text as geometry authority under this step, despite comparative prototype prose
about remembering how every number was written. A future authored-unit-intent
schema needs its own decision. Likewise, general formulas, radians and DMS are
not silently included in the ratified scalar/decimal-degree service.

GP-C04's broad store-management contract is not proof that its entire surface is
active: GP-C05 expressly withholds Manage Preferences until its operations are
implemented and proved. Catalog table cells describing eventual live consumers
are subordinate to its explicit eleven-active/45-reserved inventory. These
distinctions prevent a future authoring handoff from treating reserved identities
or historical planning status as usable capability. No request to reopen or
accept GP-CM05V is implied.

## Historical baseline versus amended Preferences authority

At `23d33510`, five further settings-route members were read completely:
`PREFERENCES_FIRST_REENTRY_AUDIT.md`,
`GP_C06_CONSOLIDATED_RATIFICATION_PACKET.md`,
`GP_C01_INTERNAL_AUTHORITY_AUDIT.md`,
`GLOBAL_PREFERENCES_ENGINE_RESEARCH.md`, and
`GP_DRAFT_SETTINGS_CATALOG_SEED.md`. PM037 was checked again against the packet.

**Tracked discrepancy: `dat-preferences-packet-supersession-rnw`.** The GP-C06
packet header says its activation and Units counts incorporate the 2026-09-05
amendment, while its body retains these conflicting assertions:

| Packet assertion | Controlling later disposition already present in PM037 and the catalog |
| --- | --- |
| Finding 2 and consolidated clause 5: an eligible aggregate Units contribution wins | The aggregate is inactive and cannot override the eight typed Units seeds in GP-CM03 |
| Finding 6 and consolidated clause 19: 58 active descriptors | 56 reserved identities, of which eleven are production-active and 45 remain reserved |
| Consolidated clause 10: search includes planned and read-only Project-policy rows | Product Global search includes production-active identities only; reserved and Project-owned rows are absent |

The issue requests explicit supersession boundaries, not a new product choice
or rewriting the historical owner acceptance. Its links to the foundation and
Preferences specification issues are related intake only; they neither claim
Preferences work nor change its hard blockers. The entire owning route remains
to be reviewed before source reconciliation or a digest update. This record
does not bless contradictory sources merely because their route hash passes.

The re-entry audit explains a concrete sequencing failure relevant to the wider
rollout: requiring complete Units GUI parity before the real Preferences window
existed created a practical cycle and pressure for a temporary surface. Its
corrective distinction is exact core prerequisite versus parity through the
real native consumer, later governed by PM039. The foundation handoff must
likewise avoid making proof through an authoring doorway a prerequisite to
building that doorway. This does not waive that proof at delivery acceptance.

GP-C01's no-Preferences/no-seed findings are explicitly a `417d16c` historical
baseline. The draft seed's 119 candidates are explicitly intake, not registry
or activation counts. Neither is a current runtime audit. The research plan's
Revision carry-forwards and old implementation ordering must be read under
PM038/039/040 and later activation amendments, not used to revive withdrawn
Revision UI or a rival roadmap. These observations establish how to interpret
the reviewed evidence; they do not claim every later consumer is reconciled.

## Typed input authority and retained technical history

At `700ec2a4`, six more settings-route members were read completely: the
GP-C03 typed-authority packet, GP-C02 external/standards research, GP-C02B peer
research, drafting-standard ownership packet, PM034, and Revision recovery
implementation inventory. External citations were reviewed as existing local
research; no fresh vendor verification or standards-conformance claim was made.

Q1/Q2/Q3/Q4 establish distinct obligations for foundation consumers:

- A remembered workflow default does not replace an explicit operation input.
  Capturing the last invocation as a persistent default requires its own
  registered descriptor; field editing cannot silently create one.
- Source family and value precedence do not confer mutation authority.
  Resolution admits eligible sources, applies eligible controls, then ranks
  ordinary values. A Project-owned rule is not overridden by a more specific
  machine or Session preference.
- Inspection is not write permission. A refused or managed value remains
  explainable, with retained contributions and typed reasons; the GUI must not
  derive authorization merely from which value appears effective.
- Q3's user-held organization grant is a deliberate Datum decision, not an
  alleged consensus of the surveyed tools. Revoking that machine grant does
  not revoke independently adopted Project law. Later PM038 exclusions still
  prevent these historical examples from activating Revision presentation.

For F2/F5/F8, the later contract therefore needs to bind each submitted value to
its field/quantity, explicit input or resolved context, owning mutation family,
and refusal outcome. This is a handoff requirement derived from existing
authority, not a new preference layer or generic override mechanism.

The drafting-standard packet ratifies ownership only. It does not choose ISO
versus ANSI, a standard edition, a usable seed schema, or the prototype's
options. `dat-adopted-drafting-standard-object-er9` remains the named schema
follow-up; exact board editing must not manufacture that object to make a
creation workflow appear complete.

PM034 separates technical journal history from human EngineeringRevision
identity. The recovery inventory at its stated `7734d87` baseline goes further:
it quarantines mandatory formal Revision coupling on normal commit/open while
retaining useful integrity primitives. It explicitly says that removing
fictional GUI rows does not establish engine decoupling. For F5/F6, later review
must locate the current authorized technical-history seam and its actual
commit/open/undo proof; it cannot count quarantined Revision tests as delivery
evidence or disable durable undo merely to avoid formal Revision behavior.

This is not a claim that the historical coupling still exists unchanged in
current code. The inventory's old Frontier table and runtime observations are
dated evidence, not task selection or a fresh implementation audit. PM038
remains controlling, and no Revision or Preferences task is claimed here.

## Complete seed/context disposition review

At `734ea079`, all 2,048 lines of
`GP_C03_PROJECT_SEED_AND_CONTEXT_DECISION_PACKET.md` were read, including every
candidate, render-gap disposition, bounded clause and owner response for Q5,
Q5A, Q11 and Q6–Q10. Its historical next-step wording does not select work now.

The packet resolves these additional F2/F6/F8 handoff obligations:

| Boundary | Settled rule and consequence |
| --- | --- |
| Preview versus genesis | Q11 §3.2.5 clause 7 says preview neither freezes nor copies a seed. Q5 §2.5 pins an immutable snapshot at genesis. A preview, intervening Global change, then creation must be assessed against the actual captured snapshot and receipt, not assumed to use the earlier preview |
| Operation cancellation | Q6 §4.5 clauses 8–9 retain explicit per-invocation input and forbid silently remembering it on cancellation or completion. A cancel proof needs to cover machine preference state as well as Project geometry/journal when the workflow could otherwise capture a default |
| Correctable refusal | Q7 §5.5 clauses 2–4 forbid semantic clamping/substitution, retain the attempted interactive value, and return typed non-interactive refusals. The last valid effective value and the uncommitted draft are different observations |
| Unknown seed inputs | Q8 §7.5 clauses 1–4 exclude unknown records from seeding, context, defaults and controls. Stored bytes or a matching label do not establish active authority; later consumer-ready activation further narrows registration eligibility |
| Explanation snapshot | Q9 §9.5 clauses 1, 3–5 and 16 require one evaluation snapshot, explicit absence/conflict, and side-effect-free explanation. A displayed value plus a separately inferred reason cannot prove provenance |

The packet still contains historical six-Units-row wording and Q10's original
Revision visibility/onboarding clauses. PM040 and the revised eight-row Units
contract control the former; PM038 and later activation amendments control the
latter. Neither the original approved Q10 response nor the packet's completed
status authorizes reviving those withdrawn product surfaces. Q5A's zero-required-
choices law remains relevant to ordinary startup, but its deferred four-checkpoint
guide is not a prerequisite or completion witness for the native doorway.

These are reviewed source obligations for the later connected workflow contract,
not executed race/cancel/refusal tests or acceptance of a new implementation.

## Product-service and creation proof boundary

At `fd96b893`, the complete product-surface contract, production acceptance
matrix and 1,246-line Preferences implementation plan were read. They supply
concrete consumer contracts, not acceptance of this foundation lane:

- Global and factory creation normalize into one engine request. MCP must name
  the source explicitly; GUI/CLI defaults are made visible. Unreadable Global
  state refuses, preserves entered name/location and offers—but never selects—
  factory mode. Factory mode neither reads nor creates a Preferences repository.
- Expected-generation checks use the full reference, not just its number. A
  reachable daemon refusal cannot trigger a second standalone writer. No-op
  writes still validate authority and expectations before returning unchanged.
- Genesis pins one eight-key seed, validates complete staged shards, and
  atomically publishes to an absent destination. Concurrent/repeated requests
  resolve the winning immutable evidence; cleanup is restricted to the caller's
  exact marked stage. Edit/reopen delivery cannot replace this with fixture-only
  Project construction or a private GUI writer.
- V1 seed receipts remain immutable; missing historical facts cannot be invented
  to relabel them V2. This is important for migration fixtures and Project Reset
  provenance, not permission to rewrite a receipt during ordinary opening.
- The production matrix is explicitly candidate evidence pending GP-CM05V.
  Its fifteen corpus cases and eight budget families are Preferences/creation
  evidence, not accepted board-edit latency budgets or proof of authored geometry.
  That owner decision remains exclusively with the other session's lane.

**Tracked discrepancy: `dat-preferences-unknown-field-code-ccc`.** The product
contract §3 assigns unknown request fields `unsupported_schema_version`, while
§7 assigns them `invalid_request`. The contract requires clients to rely on
symbolic codes, so this needs an explicit input-domain/precedence reconciliation
and cross-adapter negative vectors. It is source-level intake, not a reproduced
runtime defect or an amendment to Preferences acceptance. Preserve rejection of
unknown authority fields and trusted transport actor context; do not choose a
code here to make implementation appear conformant.

The implementation plan also explicitly states that the Units GUI adapter did
not supply a broad geometry numeric editor. Its accepted Units history cannot
close F2 native authoring by implication. F01 must connect that shared exact
service to the still-required field, mutation and ordinary-entry workflow.

## Supporting visual studies and creation timing

At `59ad6a28`, seven more settings-route prototypes were read in full textual
source: `first-run-study.html`, `guided-setup-study.html`,
`start-page-study.html`, `search-placement-study.html`,
`preferences-store-placement-study.html`, `preferences-accessibility-study.html`
and `preference-store-states-study.html`. The review included DOM, CSS and
amendments after the original closing HTML. Opaque embedded WOFF2 payloads were
elided from text output; font declarations were retained. This is not a new
render, keyboard interaction test or native accessibility result.

The first-run wizard and search-placement alternatives are superseded comparison
material, not permission to reintroduce a wizard, reserved search results or a
full-width search bar. The four-checkpoint guided setup and Start page remain
deferred. Crucially, the Start-page file also contains a separately bounded
`GP-CM03 NEW PROJECT UNITS GENESIS` form: the ordinary File → New Project
doorway must not inherit a dependency on delivering the deferred Start page.

Its G-A through G-H regions refine the seed timing obligations already reviewed:

| Event boundary | Required distinction for the foundation consumer |
| --- | --- |
| Idle Start-page preview, before a creation request | A displayed preview is not itself a frozen creation snapshot |
| Creation form submits a displayed generation as its expectation, but Global has changed | Refuse stale before genesis; retain name, location and focus; offer reread or an explicit factory choice, never silently substitute a newer seed |
| Engine has acquired the accepted seed snapshot, then Global changes | Complete against the captured eight-key seed; later Global changes cannot alter the new Project's immutable receipt |

The form describes keyboard traversal, radio-arrow selection, source/eight-value
summary, assertive refusal without focus theft and non-color disabled/selected
cues. The accessibility study supplies comparative narrow, reduced-motion,
keyboard and announcement states, but static outlines and text are not evidence
that the native accessibility tree or focus behavior works. These are separate
future native-proof observations, not a screenshot-only acceptance criterion.

The store-placement alternatives do not activate Manage Preferences. The
store-states study illustrates corruption, import collisions, migration and
restore under GP-C04; later activation boundaries still control availability.
Its corruption panel distinguishes continuing authoring with preserved damaged
machine data from a request to create using unreadable Global seeds. The latter
must still refuse under the product creation contract unless the user explicitly
chooses factory mode. “Authoring is unaffected” is not permission to silently
replace a requested creation source. Migration suggestions likewise are not
automatic value substitutions, and illustrated reserved controls/counts do not
become production descriptors.

## Revision history, standards claims and geometry proof limits

At `b1c3657b`, seven further settings-route members were read completely:
`docs/RESEARCH_TRACEABILITY.md`,
`docs/gui/DATUM_RENDER_FIDELITY_AND_DFM_GEOMETRY.md`,
`REVISION_SEQUENCING_AND_ORIGIN_RESEARCH.md`,
`PRODUCT_REVISION_ENGINE_STANDARDS_MATRIX.md`,
`PRODUCT_REVISION_ENGINE_RESEARCH.md`, `specs/PRODUCT_REVISION_ENGINE_SPEC.md`
and `specs/PRODUCT_REVISION_ENGINE_IMPLEMENTATION_PLAN.md`.

The Revision specification's §12 and implementation plan's REV-I10/I17 explicitly
mark permanent/default Revision presentation as superseded by PM038. Their
remaining historical lists, the research's earlier permanent-Navigator approval
and the human traceability matrix's old task-status summaries do not select work
or restore execution authority. The structured Frontier remains the scheduling
authority; the foundation contract must preserve the unmanaged baseline, not
require release records before an ordinary edit/reopen test.

For F5/F6/F7, the reviewed specification distinguishes technical revision/tip,
human revision, baseline and release identities. A newer library definition is
an uptake candidate, not authority to silently rebind a placed instance. A
missing impact edge is Unknown, not evidence of no impact. Historical evidence
remains tied to its frozen baseline. The foundation handoff must preserve these
identity and provenance distinctions without claiming that the suspended broad
Revision, dependency-graph or enterprise program is delivered.

The implementation plan makes writer ownership a separately required decision
if unresolved, and requires real-Project crash/recovery and unchanged mutation
semantics. This supports retaining the existing write-ownership handoff; it does
not establish that a writer decision or runtime proof already exists. Its
source-health and single-service requirements also exclude using a private
foundation editor or monolithic implementation as a shortcut.

Two evidence-strength limits matter for the later contract:

- The standards matrix distinguishes normative text, guidance, publisher scope
  and Datum policy. The sequencing research's historic ASME examples cannot
  supply missing licensed clauses or establish a current conformance profile.
  No fresh external verification was performed here, and no edition or
  certification claim is adopted from these local research summaries.
- Render Fidelity Law 1 separates canonical manufacturable geometry from
  presentation overlays. Selection glow/grid equality is not a CAM test.
  Byte-identical release reproduction, canonical geometric equivalence and
  visual equality are different proof predicates. The future DFM solver's
  illustrative ratios and operating envelopes are not numerical acceptance
  thresholds for the first exact edit, nor evidence that the solver exists.

These findings constrain future F02 examples and handoffs. They do not amend
ratified sources, refresh route digests, reopen withdrawn Revision UI or claim
runtime, licensing, visual or production acceptance.

## Integrity evidence and the single policy-check boundary

At `965b9601`, all four execution packets `REV_I00`, `REV_I02`, `REV_I03`
and `REV_I04_EXECUTION_AUTHORIZATION_PACKET.md` were read completely. Their
dated owner dispositions and completion sections are historical evidence, not
fresh execution authorization or a current runtime audit.

REV-I00 §7 records technical integrity implementation at `b329b9e`/`cf16a2f`,
including five interruption points, writer exclusion, last-complete/read-only
recovery, backup tamper refusal, restore equivalence and ordinary journaled
mutation on three named native Projects. Thus these mechanisms must not be
described as wholly unbuilt merely because an earlier packet listed gaps.
Conversely, historical passing tests do not establish today's native doorway,
numeric-edit, cancellation or reopen behavior. F6 needs current consumer proof
and reconciliation with the separately tracked writer-ownership decision, not
a second store or automatic closure of that decision from this history.

The packets establish the progression from inert records to policy evaluation
to one canonical Design integration. REV-I02's unconfigured witness and
REV-I03's no-adopted-policy witness protect absence from becoming synthetic
policy. REV-I04 then permits exactly one pre-staging coordinator preflight with
three outcomes: `UnmanagedPassThrough`, `NoEarlierControlCollect` and
`AuthorizedChangeValidated`. Normal commits, accepted proposals, Undo and Redo
must share it; GUI fields, operation validators, Project open and resolvers must
not independently introduce a policy gate.

For later F5 examples, distinguish an unmanaged Project from an explicitly
adopted policy with an actual released predecessor. The former creates no
Revision authority. The latter may quietly collect a Change and transaction
link, without reserving a label or issuing a human revision. REV-I04 Finding 3
and its final owner disposition explicitly qualify the earlier broad
no-authority-event wording: permitted collection events are not reservation or
issuance. Neither path authorizes prompts or refusal for missing earlier-control
authority; only deliberately adopted `AuthorizedChangeRequired` may select that
refusal branch, before staging and without partial Design/authority writes.

The later PM038 unmanaged-baseline correction still governs product exposure.
These packets do not reactivate Revision seed descriptors, enterprise workflows
or suspended GUI work. Their test-only signature providers are not production
cryptography, and their closed inventories do not authorize new dependencies.

## Identity decisions and corrected historical claims

At `25e20aad`, the complete Revision identity decision packet, title-block
research, REV-C07 ratification packet and REV-C01 internal authority audit were
read. The identity packet's initial “not ratified” banner is followed by dated
dispositions; it must not reopen the already-settled scheme and namespace
decisions. Those decisions still do not activate reserved Revision seeds or
override PM038's later unmanaged baseline.

The title-block research contains concrete examples of the original process
failure: it describes an existing six-state PLM machine and
`EngineeringChangeOrder`, and derives automatic revision-table rows from journal
commits. REV-C01 identifies those claims as unsupported at its `264b1a2`
baseline (REV-GAP-05/11); REV-C07 explicitly requires their reconciliation.
Future foundation consumers must cite the actual typed authority and later
dispositions, not implement these historical claims as requirements. The older
PLM-wrapper framing also cannot override the ratified no-external-master rule.

For F1/F2/F8, title-block formula proposals belong to the separate Publish
binding system. They do not authorize a formula parser in ordinary exact
geometry fields, a live clock influencing deterministic output, or a
sheet/document override of Project Working Units. For F5/F6, the audit's
distinction between authored model identity and generated evidence explains why
the accepted transaction tip must be observed separately in proof: a matching
model fingerprint alone cannot establish that no evidence transaction occurred.
The audit's test counts and missing-capability lists remain historical; later
integrity completion already reviewed must be considered before describing a
current gap.

No additional owner choice is manufactured from these dated question lists.
The remaining route review must still reconcile present consumers; this record
does not correct the original research, refresh its digest or accept a runtime
implementation merely because the contradiction was previously documented.

## Exact context, library uptake and offline proof inputs

At `7852e16a`, the full 1,033-line Revision authority model and complete
REV-C04 offline/exchange and REV-C05 impact/reproduction contracts were read.
Their candidate headers and historical gap statements remain subordinate to the
later dispositions and implementation evidence already reviewed.

The detailed model makes `ConfigurationRef::Working` exact at query time, not
a live reference that can safely be reused without retaining its model revision
and accepted tip. This reinforces the F3/F5 need to pin the selected subjects
and expected context before an edit, then refuse a stale request without
reconstructing intent from labels, filenames or current selection.

REV-C05 §§5–6 supplies concrete F7 handoff distinctions: a newer library object
creates an uptake opportunity but leaves a placed binding unchanged; lifecycle
or review-annotation changes may produce findings without replacing geometry;
missing retained pinned bytes never resolve through `latest`. Preview is
read-only, while adoption is explicit, journaled and undoable before release.
These are separate observations for future consumer proof, not evidence that
the complete uptake/impact engine is currently delivered.

REV-C04 §§5 and 7–9 separates local journal atomicity from external delivery.
An optional adapter's failure cannot roll back a completed local transaction,
and ordinary open/edit/query cannot depend on Git availability. An externally
received tree must resolve and validate in isolation before typed acceptance;
a clean textual merge is not semantic proof. F6's native Project workflow must
therefore work without making the development repository or a remote service a
hidden product prerequisite. This does not authorize distributed merge or a new
adapter in the foundation slice.

REV-C05 §§5 and 9 makes evidence freshness depend on every declared influential
input, including rules, producer, settings and environment—not solely geometry
or model identity. Its release-reproduction contract also distinguishes a new
generation of evidence from replay of an old frozen output. These are useful
source constraints for later proof design, not permission to impose the full
formal Release program on the first manual workflow or to relabel diagnostic
equivalence as byte reproduction. Missing evidence and unavailable producers
must stay explicit rather than becoming successful empty checks.

## Workspace and Publish consumer separation

At `36d9e5ba`, the complete workspace-architecture research, Publish Space
specification, PM020 and Publish visual-study brief were read. PM020 explicitly
labels its original edit-through and Sheet-local-release rationale as historical;
the 2026-08-23 reconciliation and governed Publish contract control instead.
This read does not complete the separate Publish prototype route or establish
new visual conformance.

The following boundaries are inputs to the F1–F8 consumer map:

- A Project is not constrained to one board or an imposed product hierarchy.
  One bounded board-edit fixture is a proof scope, not a new Project schema.
- Pane layout, opening, focus and specialist-editor entry are workspace state.
  Opening a Footprint or Symbol beside its invoking editor does not decide
  whether the asset is shared or Project-local, nor authorize changing it.
  F7 must bind that separate library authority explicitly.
- Navigator selection is distinct from opening: single-click selects;
  double-click/Enter opens beside. Scrolling the Navigator must not pan an
  editor. These are native behavior observations, not facts proved by a static
  fixture-driven tree or a screenshot of two panes.
- Continuous schematic Design has no physical publication page. Legacy
  schematic `Sheet*` identities and addressing require an explicit migration
  contract; a label rename or alias is not that migration. The bounded board
  workflow cannot claim to have completed this separate schematic work.
- Design driving and reference dimensions differ. A driving constraint that
  over-constrains the model refuses and offers an explicit reference alternative;
  it must not silently change intent. Publish dimensions never drive geometry.
  This is not authority to implement a dimension solver in the foundation slice.
- Publish's signed-nanometer Sheet coordinates and reduced rational scales
  belong to physical composition. They neither redefine board coordinates nor
  make Publish media, title blocks or production acceptance prerequisites for
  ordinary native board creation/edit/reopen.

The brief's adjacent-versus-retarget candidate is closed by PM020's later
OR-3 carry-forward; it is not a new owner question. PM020 OR-7 also expressly
withholds ratification of illustrative responsive-collapse priorities. A later
implementation must use the controlling responsive-shell contract, not promote
every visual example to approved behavior. None of these observations changes
the Preferences-owned window contract or authorizes prototype edits.

## GUI action ownership and historical visual inventories

At `97dcdd5d`, the complete GUI Design specification, REV-C06 visual brief and
`prototype-disposition-sheet.html` textual source were read. The disposition
sheet is explicitly a derived reference regenerated on 2026-08-29. Its
“safe for any slice to build against” and “nothing here gates development”
language cannot authorize execution or override later PM038/039/040 decisions.
In particular, historical approved markers for Start/Guided Setup and Revision
studies do not establish present activation or acceptance. Conversely, a
historical marker on a shell study does not cancel its specifically retained
visual rules in the controlling GUI contract.

The GUI specification contains broad summaries that require the more specific
authorities already reviewed: “selection is engine-level” cannot turn selection
into journaled Design state or collapse same-identity and related-context
projections; “every [menu] item emits a typed Operation” cannot turn its own
explicit `gui_local` navigation/view actions into mutations. F3/F5 examples must
identify the action class and expected journal effect separately. Tool starts,
previews, navigation, cancellation and committed edits are not interchangeable
success events.

The Console sections require output-only, non-focusing feedback, with committed
operation history projected from the journal. They record prior closure claims
withdrawn when source-health and acceptance evidence failed. A Console success
echo therefore cannot prove an edit committed, and its old closure/test counts
cannot replace a current native workflow witness. Refusal text, typed identity,
focus preservation and durable journal outcome are separate observations.

The Revision brief likewise says native AT-SPI behavior needs its own proof;
HTML accessibility prose is not native conformance. Its early modal-confirmation
permission is narrowed by the later Q5-B-amended in-pane arm/disarm disposition,
and the entire suspended Revision UI remains under PM038. Neither pattern is an
automatic confirmation rule for ordinary reversible board edits.

`shell-parity-comparison.html` was inspected in textual form with its embedded
PNG bytes elided. Its historical percentage and suggested explanation are not a
fresh rendering finding. The comparison images still require visual inspection;
this file is deliberately not counted as fully reviewed below. No comparison,
prototype disposition, golden or authority digest was blessed or modified.

## Remaining review coverage

WDQ-F01 remains in progress. The `workspace-documentation-and-revision` route
contains 79 files. PM035 through PM040, `PROJECT_PREFERENCES_SPEC.md`, the shared
Units requirement, GP-C04, GP-C05, the V1 descriptor catalog, the five historical
baseline files, the six typed-authority/history files, and the complete
seed/context packet, three product-surface/acceptance files and seven supporting
visual studies, plus the seven Revision/geometry/traceability members listed
above, the four execution packets and four identity/title-block/audit members
plus the detailed authority/offline/impact contracts and four workspace/Publish
members plus the GUI contract, Revision visual brief and prototype inventory
have been read completely (58 files); the other 21 files remain to
review. No complete settings route reconciliation is claimed. The remaining sources and consumers must be
reviewed before final F1–F8 adjudication or specification edits. Later storage,
mutation, identity and authoring
authority must also be checked where it resolves the historical questions above.

WDQ-F02 still owes concrete values, boundary cases, transaction timelines and
consumer bindings. WDQ-F03 still owes the reviewed contract and exact proposed
prerequisite handoffs for owner disposition. The broader goal also owes explicit
enrollment and verified adoption beyond the pilot; this foundation task does not
close that goal. No new dependency, runtime implementation, native proof,
independent replay, visual-truth change or product acceptance is claimed here.
