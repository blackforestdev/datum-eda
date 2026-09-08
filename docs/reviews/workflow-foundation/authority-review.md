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

## Remaining review coverage

WDQ-F01 remains in progress. The `workspace-documentation-and-revision` route
contains 79 files. PM035 through PM040, `PROJECT_PREFERENCES_SPEC.md`, the shared
Units requirement, GP-C04, GP-C05, the V1 descriptor catalog, the five historical
baseline files, the six typed-authority/history files, and the complete
seed/context packet and three product-surface/acceptance files listed above have
been read completely (26 files); the other 53 files remain to review. No complete settings
route reconciliation is claimed. The remaining sources and consumers must be
reviewed before final F1–F8 adjudication or specification edits. Later storage,
mutation, identity and authoring
authority must also be checked where it resolves the historical questions above.

WDQ-F02 still owes concrete values, boundary cases, transaction timelines and
consumer bindings. WDQ-F03 still owes the reviewed contract and exact proposed
prerequisite handoffs for owner disposition. The broader goal also owes explicit
enrollment and verified adoption beyond the pilot; this foundation task does not
close that goal. No new dependency, runtime implementation, native proof,
independent replay, visual-truth change or product acceptance is claimed here.
