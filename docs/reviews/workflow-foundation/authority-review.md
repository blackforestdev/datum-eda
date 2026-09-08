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
this file was not yet counted at that review checkpoint. The subsequent
embedded-image inspection is recorded below. No comparison,
prototype disposition, golden or authority digest was blessed or modified.

## Excluded agent authority, refusal feedback and retired teaching

Six more route members were read completely, including CSS, all body regions
and trailing dispositions: `agent-authority-study.html`,
`agent-authority-scenarios.html`, `agent-authority-threat-model.html`,
`agent-blocked-action-alerts.html`, `clay-rows-decision.html` and
`revision-carryforward-study.html`. Only opaque embedded WOFF2 payloads were
elided from the read output. This is source review, not native visual or
accessibility proof. No Claude-owned file was changed.

The four agent studies distinguish read/propose, applying an already-approved
proposal, and specifically scoped unattended actions. Their August 28 owner
dispositions require tool/subject/rate/expiry scope, no inherited unattended
grant on a new launch, organization restrictions that cannot enlarge authority,
and deliberate human grant management in Preferences. The studies explicitly
do not protect design confidentiality against the launched harness's filesystem
or network access. A delivery gate must not describe mutation authorization as
an IP-exfiltration sandbox, nor infer agent readiness from a read-only query.

These are excluded future surfaces, not active Preferences capabilities:
PM037 and the descriptor catalog still require dedicated mechanism/security
review and separate activation. The route's addendum 23 also states this
explicitly. Visual approval does not ratify grant schemas or authorize their
implementation in the current Preferences lane.

There is a concrete supersession hazard, captured as intake
`dat-agent-authority-visual-supersession-rh3`. The authority study's sections 1
and 3 still say release capability is structurally impossible at every level,
while its disposition permits deliberate future opt-in from a closed default.
Section 4 still calls the decisions open; threat-model T3/T4 and its closing
summary still call the identity-binding and restriction-only rules undecided.
The scenario D1-B also depicts an agent signing an attestation: granting release
capability must not silently authorize that separate capability. The bounded
Claude handoff is to label/reconcile these exact historical regions while
preserving dated dispositions, PM034/PM037, attestation restrictions, issued
immutability and Project-policy boundaries. Proof must distinguish current
authority from comparative alternatives across all four studies and leave the
excluded catalog unchanged. No handoff completion or source digest refresh is
claimed here.

The blocked-action study supplies useful refusal observables: name the tool,
reason, what did not happen and where the person can act; do not steal focus or
grant from the output-only Console or a notification. Its future persistent
record counts repeats, rate-limits notifications and makes Clear affect only
the record. Expiry's return to propose is not successful execution of the
original request. Its illustrated automatic proposal queue is not authority to
invent a proposal endpoint or silently retry a failed manual operation. F5/F8
consumer proof must distinguish refusal, proposal creation and committed Design
change; a status message alone proves none of their durable effects.

The clay-row study marks only grid-preset vocabulary decided; crosshair and
opening-layer alternatives remain open. Even the grid visual disposition does
not activate a descriptor excluded by PM037. Preference vocabulary must not
replace PM023's shared grid/tier ownership, and transient view changes must not
be silently promoted into durable defaults or Project policy. Exact grid-basis
authority still needs the remaining units/grid studies before F4 adjudication.

The carry-forward study explicitly retires managed Revision visibility and
mandatory teaching replay under PM038. Old approved/open language lower in
that file is retained comparison, not permission to restore organization-forced
Revision chrome in an unmanaged Project. Its separate guidance example stays
beside the release confirmation, preserves focus and never gates issuance.
Neither that formal-release ceremony nor its onboarding state is a prerequisite
for the ordinary manual edit/reopen foundation workflow.

## Grid presets, exact readouts and connectivity evidence

`grid-basis-candidate-d.html` and `units-and-grid-model.html` were read
completely, including CSS and both later Units amendments, with only opaque
embedded font bytes elided. Candidate D is retired: the later study explicitly
rejects pitch-derived footprint grids and its proposed metric-native library
module. Neither is a foundation requirement. The later study retains the
August 29 grid-set/default disposition, separately from PM040's supersession
of per-person live display ownership and the still-unratified authored-expression
persistence. Existing Projects use Project Working Units; switching a reading
must neither move geometry nor implicitly quantize it.

The owner-dispositioned preset values in section 7B, not candidate D or earlier
illustrative section 5, are the relevant visual reference for F4:

| Set | Pitches shown | Default/qualification stated |
| --- | --- | --- |
| Metric | 2.5, 1.0, 0.5, 0.2, 0.1, 0.05 mm | Default set; board 0.5 mm, schematic 2.5 mm |
| Mil | 100, 50, 20, 10, 5, 1 mil | Optional set, not imposed by a display change |
| Imperial-derived metric | 2.54, 1.27, 0.635, 0.254, 0.127 mm | Never the default and never suggested |

This does not activate the excluded grid descriptor or supersede PM023's
shared profile/snap ownership. The study's historical claim that the fine-grid
descriptor already ships is not current production-active catalog evidence.
Likewise, a named preset and a rounded label are not an exact edit buffer:
0.5 mm is exactly 500,000 nm and 2500/127 mil, not exactly 19.685 mil. The shared
Units focus/no-op contract controls how a finite display is distinguished from
canonical input. No manufacturing precision or standards-conformance claim is
inferred from the nanometre representation or the study's peer comparisons.

F4/F7 must explicitly resolve a source conflict captured in
`dat-grid-connectivity-authority-5s6`: the study's section 2 relies on exact
coordinate coincidence, citing `docs/SCHEMATIC_CONNECTIVITY_SPEC.md:92`, while
UVT section 3.3 says connectivity survives quantize because nets are UUID/net
addressed, not coordinate-coincident. These may describe different layers but
cannot substitute for a specified attachment/update contract. The full
connectivity/authoring review must distinguish net identity, geometric
connection formation and connected-wire movement. Preserve PM023's exact
committed coordinates, UVT's grid-then-object override, screen-pixel eligibility,
target filtering, explicit align `reference: grid` and whole-batch refusal;
do not choose an electrical model from whichever paragraph is closest.

The study's two manufacturing allegations were checked against current source,
not accepted wholesale. `dat-export-signed-decimal-parse-tc3` records the
engine helper's signed-fraction arithmetic and the CLI helper's negative-zero
loss. Inspected callers use a fixed outline aperture or parse aperture sizes;
they do not establish the claimed corruption of negative board-coordinate
verification. `dat-excellon-header-dialect-yph` records the observed `METRIC,TZ`
header with decimal coordinates as a dialect research question, not a reproduced
CAM scaling failure or authority for the proposed one-line fix. Both require
owning-lane work and independently grounded proof. No runtime files, manufacturing
contract, protected HTML, source digest or Preferences acceptance were changed.

## Project Working Units visual target boundary

`project-preferences-category-study.html` was read completely, including its
two long minified P-A–P-J lines, CSS/responsive rules and original category
comparison. The target explicitly remains open for owner review; the historical
category inventory below its divider is clay. A schema's existence or a count
of code references does not authorize a populated category, and its historical
Global read-only mirror claim does not override PM039's separate doorway.

The target draws one Project Preferences terminal command, disabled with a reason
without an open Project; Units belongs inside its owned input-modal window,
never the Navigator or another submenu. Its eight independently exposed controls
retain per-quantity Automatic resolution and cross-system disclosure. Reset
reads immutable seed/migration provenance, submits a journaled undoable Project
change, and never reads current Global defaults or changes the receipt. Missing
receipt data makes Reset unavailable rather than guessed.

P-G makes stale generation an explicit refusal followed by re-read, not a retry
of the rejected edit. P-E/P-F mark all eight unreadable controls independently
with text, glyph and dashed outlines. P-E illustrates session use of recorded
seed values while preserving the damaged profile; the full failure contract
must also cover an unreadable receipt, not infer an unconditional fallback from
this single case. The design-authoring-preserved claim still needs native proof.
P-I/P-J specify rail/search/control order, contextual Reset placement, scoped
Escape handling and per-selector accessible states. The 980-pixel responsive
rule changes the rail/row layout; HTML labels and inventory tables are not proof
that the native accessibility tree or narrow layout meets those requirements.
No open visual target was accepted, edited or promoted into GP-CM05V scope.

## Revision entry, evidence presentation and historical shell images

Four more textual sources were read completely: `revision-ux-study-index.html`,
`project-preferences-revision-gate.html`, `revision-contextual-entry.html` and
`revision-ux-evidence-study.html`. CSS and trailing status statements were
included; only opaque font bytes were elided. The ten embedded PNGs in
`shell-parity-comparison.html` were also extracted read-only and visually
inspected, completing that member's previously textual-only review. No files
were generated for extraction and no prototype was modified.

The contextual-entry study explicitly separates technical history from formal
Revision adoption. A new unmanaged Project retains journal, transaction tips
and undo without creating EngineeringChanges, baselines, releases or controlled
documents. Subject-local actions preserve the Design pane and explain unavailable
baseline-dependent actions; requesting a result does not add permanent Navigator
groups. Where controlled records are eventually browsed remains a stated owner
decision, not permission to invent an entry point for foundation delivery.
The Project Revision gate itself remains clay and does not activate its policy
dropdown or Global Revision seeds.

`dat-revision-visual-supersession-3c1` captures three exact owning-lane
reconciliation targets: the index's Q10 row still presents withdrawn managed
visibility/teaching replay as approved; the Project gate says unmanaged means
"nothing recorded" and retains a question about an already-withdrawn nag;
and evidence UX-V12 still describes final-issuance modal focus despite the
ratified in-pane arm/confirm boundary. Preserve dated dispositions and technical
history, label historical alternatives and reconcile those regions through
Claude. This intake neither edits HTML nor changes PM034/PM038 authority.

UX-V10's selected Q6-A-amended design makes Sources, Producer, Invocation,
Environment, Outputs and Attempts permanent counted categories even when clean.
Missing, unavailable, mismatched or failed evidence stays expanded. Full
immutable detail opens beside the summary; it does not replace it. Reproduction
and authenticity are separate verdicts; CanonicalEquivalent is diagnostic,
not satisfaction of byte-identical specified outputs. A failed later attempt
does not mutate the issued record. This is an important limit on broad rollout
claims: an empty findings list cannot substitute for accounted-for evidence,
and verified bytes alone cannot assert owner acceptance or signer authority.

UX-V11 keeps absent Git an ordinary capability state, mirror failure separate
from local authority and external changes behind semantic review. UX-V12
explicitly requires later native AT-SPI proof; static keyboard/state examples
are behavioral references only. Recoverable truncation, contextual keyboard
entry, return focus, non-color state labels and bounded announcements remain
consumer obligations, not functionality proved by reading the prototype.

In the shell image pairs, schematic junctions, symbol interiors and strokes
visibly differ; board-crop stroke edges differ; the terminal tab gains a filled
active state and accent rule; and the revision text changes. The full frames
also show different grid patterns and schematic presentation. These are
historical embedded captures, not fresh runtime observations. Pixel crops do
not establish electrical correctness, geometry identity, stable-input revision
determinism or which change the owner wants. The page's final suggested blanket
bless is therefore not acceptance authority. No current defect, visual golden,
route digest or pilot acceptance was inferred from the historical percentage.

## Revision identity, impact and release source closure

The remaining five Revision studies were read completely:
`revision-identity-study.html`, `revision-authority-study.html`,
`revision-ux-impact-study.html`, `revision-ux-release-study.html` and
`revision-ux-shell-study.html`. This includes all CSS, inline SVG title-block
examples, comparative candidates, selected amendments and final indexes. Only
opaque embedded font bytes were elided. No prototype was changed or native
conformance asserted.

The identity study retains original no-default/candidate language despite its
ratification badge. Later REV-C03 dispositions, PM034 and the shell study's
preserved-disposition list control the settled profile, provisional reservation
and per-CI boundaries; the original candidates are not fresh owner questions.
ASME sequence examples remain licensed-text-gated, not standards-conformance
proof. Short title-block labels require recoverable namespace, scheme and
allocating-event context; a baseline manifest, BuildIdentity, member revision
and Release cannot stand in for one another. A previewed next label allocates
nothing, and identity-free successor collection is not issuance.

The authority study's final index resolves per-CI ownership and the separate
ControlledDocument chain even where earlier captions still say no candidate
selected. All Sheets retains Sheet bodies; editable Publish Sets supply ordered
references, not issued identity. Renaming or reordering a set cannot rewrite an
issued document. Referenced-set deletion refuses with a retarget/removal route,
rather than erasing downstream authority. V10-C preserves three typed standing
facts and profile-governed terminology; a display label is not an arbitrary
mutable lifecycle string, and redelivery does not restore withdrawn authority.
These are consumer boundaries, not added prerequisites for native board entry.

Impact UX-V4 keeps Unknown and graph scope visible even when zero/complete.
Both Affected and Unaffected need witnesses; a missing evaluator cannot become
a clean result. UX-V5 separately presents pinned library identity, read-only
uptake preview, compatibility and freshness. A new library version does not
rebind a placed instance, and a producer-version change can invalidate evidence
without a geometry edit. UX-V6 compares stable identities, so rename is not
delete/add; unaccounted controlled differences have an explicit resolution
route. The full controlled-release graph is not claimed by the bounded F7
library/connectivity handoff.

Release UX-V7 distinguishes candidate regeneration from reproduction of an
issued output. Reuse needs current-input proof; regeneration creates successor
evidence, never refreshes history in place. UX-V8's selected amendment keeps
nine section headings/verdicts/counts visible, with deficient details not
collapsible. Covered-digest changes invalidate affected approvals. UX-V9's
selected amendment is the two-step in-pane bar: first arm and show exact
consequence, then confirm; Escape or outside interaction disarms. Historical
modal candidates are not the chosen mechanism. The factory manual edit path
does not acquire this irreversible-release ceremony.

The shell study explicitly withdraws default Revision groups and its old hide
toggle while retaining contextual entry. Complete Q4 panes keep record meaning
and section order stable across profiles; changing an obligation is not changing
the record's identity. Its baseline-view refusal, quiet post-release collection
and regulated-policy refusal illustrate different contexts. Foundation proof
must identify the actual context instead of copying one of those behaviors
universally, especially into an unmanaged Project.

## Final Preferences visual-source review

At `5dd0109f`, `preferences-ux-study.html` and `preferences-window.html`
were read through EOF, including CSS, descriptor metadata and comparative-demo
JavaScript. Opaque embedded font payloads were elided; no new render or native
accessibility result is claimed. This completes reading all 79 members of the
`workspace-documentation-and-revision` route, not reconciliation of its tracked
contradictions or acceptance of another lane's implementation.

The UX study's PX-V1 tiled-pane candidate is not the later native owned-window
contract. PX-V6 precedence is explicitly illustrative; PX-V7's copy-once receipt
law survives, but its Revision/template seed examples do not activate excluded
descriptors. PX-V9 preserves refused input and reports degradation; PX-V10 keeps
unknown data inactive instead of guessing its meaning. PX-V11's shared resolver
answer remains applicable, not its withdrawn Revision-visibility example.
PX-V12's managed teaching visibility is superseded by PM038. The bounded Claude
reconciliation target is PX-V3/PX-V4/PX-V6/PX-V11/PX-V12: identify historical
Revision examples as superseded, preserving dated dispositions, technical
history and non-gating release guidance. No new default Revision chrome,
descriptor or approval is requested. This extends the same historical-visual
problem recorded in `dat-revision-visual-supersession-3c1`.

The window distinguishes completed GP-F05 foundation evidence, accepted U-A–U-H
Units evidence, the C-A–C-H consumer-ready target and the historical comparative
catalog. Those regions are not interchangeable acceptance packets. C-F explicitly
separates immediate persistence from effect: three Appearance consumers apply
live; eight Units descriptors seed future Projects only. Existing Project bytes
must remain unchanged by a Global edit. This is not proof of field entry,
board movement, Project-policy editing or durable edit undo.

The I/J window contract is owner-relative input modality, never desktop-wide
always-on-top. Closing discards search, explanation, choice, notice and internal
focus, but preserves committed values. Reopening starts at Appearance with
navigation focused. Search activation targets the setting-name action, not an
accidental value change; choice/explanation dismissal precedes search clearing
and window close. C-G preserves both sections and usable controls when narrow.
C-H's announced saved state follows the write, not optimistic interaction.
Role/name inventories and static non-color examples still require native proof.

The comparative JavaScript deliberately searches historical/reserved rows,
computes illustrative explanations from DOM text, and blurs search on Escape.
It is not a reusable resolver, active catalog, keyboard contract or production
implementation template. C-E and the product contract instead require active-only
search and preserved search focus after clearing. Likewise the older U-A angle
label `0°` is not zero resolution: the current C-B label is `1°` for `decimal_0`.
No prototype edit, current Preferences disposition or authority digest refresh
was made by this review.

## Consuming-owner selector verification

Fresh named `project_status.py details` invocations at this review baseline
established the following operational evidence. These are read-only handoff
checks, not claims or a replacement for the canonical task selector.

| Consumer | Observed result | Handoff consequence |
| --- | --- | --- |
| GLOBAL-PREFERENCES-COMPLETION | Success; GP-CM05V owner decision, no live claim | Preserve that separate acceptance boundary; no action or approval in this lane |
| PROJECT-PREFERENCES-SPEC | Success; blocked, planning, PPS-C01, no live claim | Contract consumer exists; no execution or prerequisite change implied |
| GUI-SURFACE-SPECS | Success; planned, planning, SURFACE-S01, no live claim | Surface-contract consumer exists; no claim or schedule change implied |
| UVT-S5A-BUILD | Exit 1: `has no completion plan` | Owner/selected-step verification incomplete; do not reconstruct an answer |
| GUI-WRITE-PATH | Exit 1: `has no completion plan` | Owner/selected-step verification incomplete; do not reconstruct an answer |
| NATIVE-AUTHORING | Exit 1: `has no completion plan` | Owner/selected-step verification incomplete; do not reconstruct an answer |

`dat-foundation-consumer-plans-9at` records the three failed consumer contracts
as specification intake related to this lane. Their remediation needs exact
governed completion outcomes, requirements/evidence, owner boundaries and passing
named selectors; tests must reject missing contracts instead of inventing steps.
No hard dependency was added and no consuming-owner acceptance is asserted.
This is an actual rollout gap, independent of GP-CM05V. F01 cannot claim complete
consumer verification while these selectors fail.

## Native authoring and storage clause adjudication

At `f95db191`, PM003, PM007, PM017, PM018, the GUI Write-Path Plan,
both schematic and PCB authoring contracts, and both connectivity documents
(`docs/` rationale and controlling `specs/` contract) were read completely.
None of these nine files is a member of an evidence route in the inspected
manifest. This does not waive review of their linked authorities: the GUI
write-path's PM019 editor-shell route and remaining storage/library authority
are still outstanding. No historical implementation inventory was promoted to
fresh runtime evidence.

| Clause / foundation concern | Authority classification and bounded disposition | Required handoff evidence |
| --- | --- | --- |
| PM018 Decision and Invariants; F6 genesis | Ratified: genesis is engine-owned and non-journaled; later accepted Units receipts are separate genesis evidence, not a fabricated zero-operation transaction | New Project proof distinguishes publication/receipt from the first authored edit; no undo entry, model-revision production or journal count attributed to genesis |
| PM007 persistence boundary; F5/F6/F8 workspace | Project source, workspace composition and volatile entry/drag state are distinct; later PM021/PM039 govern their detailed UI exceptions | Closing or rearranging a pane cannot delete source; restoring stale workspace references must resolve current identities, never replay edits or roll back source |
| GUI Write-Path W4/P0 and PM017; F2/F5 dispatch | Required typed GUI action to daemon/native commit or proposal, then resolver refresh; registry enumeration is not proof of a callable native edit | Native input, typed request, journal result and refreshed stable object must be observed together; a terminal command string or successful catalog query cannot substitute |
| PCB contract §3 and Current proof slices versus UVT §2.2.14 S5-C06; F3/F4/F5 | Contradictory: the PCB text permits locked-member skipping; later atomic-refusal law explicitly covers align and every selection-derived surface | Reconcile to complete preflight and whole refusal for locked/stale/incompatible/constrained/invalid members, with zero journal effect, exact blocker report and unchanged selection; do not bless the legacy partial result |
| Schematic place-symbol / end-to-end proof versus PM001 durable undo; F5 | Contradictory: restoring geometry is not restoring revision history. Compensating undo is append-only and advances object revisions, so a blanket requirement that model_revision reverts is incorrect | Assert exact restored domain values and stable identities separately from new transaction, object revisions and resulting model hash; test after reopening |
| Schematic end-to-end proof versus run-erc §10; F5/F7 | Contradictory: the sequence requires one transaction at every step although run-erc explicitly emits no Operation | Count authored mutations only; check evaluation has no authored-source mutation. Any durable check-evidence path needs its own classification, not an invented source edit |
| Both authoring contracts' grounding / unsupported / open-question sections; F2–F7 | Historical status conflicts with later sections describing landed native journal and hierarchy support; controlling CLAUDE also declares native write convergence complete | Reconcile dated grounding against current canonical status and inspect implementation evidence for the bounded handoff; retain genuine normalization/GUI gaps without rescheduling retired private-writer migration |
| Schematic draw-wire §4 versus connectivity rationale §§1–2; F4/F7 | Contradictory lower-level instructions: auto-junctions at crossings/Ts versus never implicitly create a junction. The formal spec distinguishes explicit crossing joins and source-format semantics | Resolve endpoint-on-segment T, crossing without junction, crossing with explicit join and pin attachment separately; imported-format exceptions cannot silently set native gesture policy |
| Schematic draw-wire §4; F5/F7 | Unanswered transaction boundary: “double-click/Esc to finish” does not say which previously clicked segments are already committed or what Escape discards | Specify exact before/after journal counts for start, vertex, finish and Escape. Preserve no-write cancellation for an uncommitted preview rather than treating every Escape as success |
| Connectivity rationale §4 versus formal spec §5 and PM003 Identity; F7 | Conflicting identity explanation: UUID-from-name rationale cannot override names-as-attributes and stable NetId/NetAnchor authority | Separate connectivity formation from durable identity; rename, merge/split and connected movement need exact identity and topology assertions, not visually touching endpoints |

`dat-authoring-clause-authority-blm` captures the atomic-refusal, undo/check
and stale-grounding reconciliation. The junction, gesture and net-identity
observations extend the bounded F4/F7 question in
`dat-grid-connectivity-authority-5s6`; they do not select a replacement
connectivity model or alter a Claude prototype. PM003 distinguishes electrical
intent, physical realization, authored relationship kind and derived relationship
status: moving board geometry cannot silently rewrite electrical intent, and
merely placing a package does not prove its unrouted nets are implemented.

The existing writer-lock issue `dat-project-write-ownership-lock-0ne` is still
open and explicitly requires a numbered decision before GUI-WRITE-PATH. Its
description identifies a journal-tip check/rename race, not a ratified choice
between daemon-only ownership and per-process locking. The foundation proposal
must present that exact choice with stale/second-writer and crash consequences;
it cannot copy the Global Preferences writer lease into Project authority.

## Editor-shell and storage review; PM012 status reconciliation

At `cec92365`, the complete eight-file `prototype-editor-shell-and-panes`
route was read: PM019, PM021, GUI Design and Conformance specifications,
Publish visual brief, and board, schematic and workspace prototypes including
their DOM/CSS and interaction code. Its unchanged digest is
`33e4ef5a297c1fbc52cd93264cbd87e65b53a7153a86277d7f639380ec09f0af`.
PM000D was also read completely. No fresh native or screenshot proof was run.

| Clause | Disposition for the bounded consumer contract |
| --- | --- |
| PM019 typed GUI dispatch | A real native action must reach the engine builder, commit/proposal and resolver refresh; a terminal string or registry entry is not editor proof. |
| PM021 focus and layout | Focused leaf supplies editor context; gutter resizing is workspace-only. Tile/Zoom/Stage round trips preserve the view tree and do not journal source changes. |
| Conformance §0 versus controlling CLAUDE construction-reference rule | The old image-only instruction is superseded: inspect DOM/CSS and translate construction into native primitives while respecting governed tokens. Return any HTML change to its visual-truth owner. |
| Conformance §0.1/§2.10/G9–G10 | A historical build-to-build screenshot baseline detects visual change, not usable editing or owner acceptance. No baseline refresh or retrospective acceptance follows from this review. |
| Conformance §8 R1–R4 scheduling versus read-only S5A | Mutation-path proof belongs with the mutating consumer, not an implicit expansion of read-only S5A. A repaired consumer completion plan must preserve this boundary. |
| Board/schematic/workspace prototype interactions | Focus classes, collapsibles, painted coordinates and static pane examples do not prove native exact entry, recursive resizing, resolver identity or journal behavior. Later Units/grid and continuous-schematic decisions govern historical examples. |
| PM000D Transactions and Revision Classes | Before the durable journal commit point, discard uncommitted staging; after it, roll forward or expose explicit recovery. Undo restores domain values through a new compensating transaction, not by rewinding model history. |
| PM000D identity and resolver | Net rename preserves NetId; split preserves the anchored group's identity; merge records the deterministic survivor. Fatal coherence failure exposes read-only diagnostics, not a partially authoritative editable model. |
| PM000D unresolved layout and writer forks | Single-writer intent does not select its locking mechanism or final partition layout. Preserve the existing writer-lock decision issue; do not copy Preferences storage authority. |

The PM012 blanket draft-header/doctrine-class mismatch is now reconciled in
its own decision and governance entry, with explicit clause limits and a
current/target Progress row. PM012 is now listed in this lane's governing docs.
It has no owning evidence route in the inspected manifest. This is the
WDQ-F01-requested authority clarification, not a new ratification: PM001's
settled mechanisms and the August workspace correction remain intact, while
fixtures, numerical budgets and genuine owner questions remain unresolved.
No inventory shape was added, so no parity registration changes are required.
Earlier entries above describe the pre-clarification baseline; their statement
that this particular header mismatch is pending is superseded by this section.

## Remaining review coverage

WDQ-F01 remains in progress. The `workspace-documentation-and-revision` route
contains 79 files. PM035 through PM040, `PROJECT_PREFERENCES_SPEC.md`, the shared
Units requirement, GP-C04, GP-C05, the V1 descriptor catalog, the five historical
baseline files, the six typed-authority/history files, and the complete
seed/context packet, three product-surface/acceptance files and seven supporting
visual studies, plus the seven Revision/geometry/traceability members listed
above, the four execution packets and four identity/title-block/audit members
plus the detailed authority/offline/impact contracts and four workspace/Publish
members plus the GUI contract, Revision visual brief and prototype inventory,
and the six agent/clay/carry-forward studies, both grid studies and the Project
Working Units category study, four Revision entry/evidence members and the
completed shell image comparison, and five Revision identity/authority/UX
studies above, and the two Preferences studies just reviewed have been read
completely (79 files). No complete settings route reconciliation is claimed:
tracked clause conflicts still need their owning-lane disposition. Later storage,
mutation, identity and authoring authority must also be checked where it resolves
the historical questions above. The three failed consuming-owner selectors need
governed completion contracts before F01 can close.

WDQ-F02 still owes concrete values, boundary cases, transaction timelines and
consumer bindings. WDQ-F03 still owes the reviewed contract and exact proposed
prerequisite handoffs for owner disposition. The broader goal also owes explicit
enrollment and verified adoption beyond the pilot; this foundation task does not
close that goal. No new dependency, runtime implementation, native proof,
independent replay, visual-truth change or product acceptance is claimed here.
