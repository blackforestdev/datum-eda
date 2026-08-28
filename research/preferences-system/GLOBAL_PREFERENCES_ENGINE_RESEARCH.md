# Datum Global Preferences Engine Research Plan

> **Status:** Scheduled research and specification plan; no implementation is
> authorized.
>
> **Tracker:** `dat-global-preferences-engine-qcv`.

## Purpose

Datum needs one typed preferences authority that remains simple for an
individual designer while scaling to organization-managed environments. It
must distinguish machine-local presentation choices from governed Project
policy, show which scope supplied an effective value, preserve unknown data
through version changes, and seed new Projects without silently rewriting
existing Projects.

This plan incorporates the Product Revision Engine carry-forwards: revision
scheme selection, provisional watermark presentation, BuildIdentity hierarchy,
ISO revision/status projection, prototype-to-production namespace policy,
successor-work control, and optional Revision-system visibility. These are
inputs to preferences research, not permission to alter Product Mechanics 034.

## Named owner carry-forward requirements

<!-- OWNER-REQUIREMENT:GLOBAL-PREFERENCES:MANAGED-REVISION-VISIBILITY -->
### Managed Revision Visibility

`Hide revision system` is a user presentation right in unmanaged contexts, not
an unconditional override of organization policy. Teaching, onboarding, and
governed environments may apply typed managed policy that pins Revision-system
visibility **on**. The UI must disclose the effective value, managing scope,
and reason; it must not pretend the control is user-editable when policy owns
it. Pinning visibility changes presentation only—it creates no new revision
authority and cannot weaken the first-class workflow that ignores Revision
authoring where earlier control was not explicitly adopted.

The Q3 owner disposition further requires a typed, revocable, user-held
`AuthorityRelease`: organization policy, including a Revision-visibility pin,
is inert on a machine until the user releases a sufficient level. Enrollment
never grants authority. Governed-Project law remains independently binding on
the Project and is neither granted nor revoked through this machine dial.

<!-- OWNER-REQUIREMENT:GLOBAL-PREFERENCES:USER-HELD-AUTHORITY-RELEASE -->
### User-held organization authority release

Organization directives use `Recommend`, `Constrain`, `Pin`, and `Lock`, but
become eligible only within an explicit user grant of none, recommendations
only, bounded, backside management, or full management. The grant and its
revocation are typed, attributed facts; excess requests remain visible and
inert in Preferences and never prompt. Revocation restores retained user
values, while any organization withdrawal of resources is separately
attributed. Backside management is the expected enterprise posture; full
management is an expressly released maximum. Personal accessibility keys are
the only descriptor-level organization-control carve-out class and require
named justification in the v1 catalog.

<!-- OWNER-REQUIREMENT:GLOBAL-PREFERENCES:FIRST-RELEASE-ONBOARDING -->
### First-Release Onboarding

A user's first Release is a designed onboarding moment, not an accidental
encounter with unexplained controls or a refusal. Preferences and presentation
must specify discoverable, accessible, context-preserving first-release
guidance that explains the two clocks, what will be issued, what remains
editable, the consequences of confirmation, and where evidence can be reviewed.
It must help without silently changing policy, bypassing the approved arm-then-
confirm boundary, inventing revision identity, or turning routine later
Releases into repeated tutorials. The exact trigger, completion memory, reset,
managed replay/requirement, and accessible rendering remain GP-C03 through
GP-C05 design questions.

Both requirements carry the owner's ratification-day direction that Datum
should help rather than harm and should actively break poor documentation-
control habits without obstructing Design authoring.

<!-- OWNER-REQUIREMENT:GLOBAL-PREFERENCES:DOMAIN-PEER-ADDENDUM -->
### Domain-peer architecture addendum

Before GP-C03 Q3, a bounded GP-C02B addendum must survey SOLIDWORKS,
SOLIDWORKS PDM administration, Altium Designer/managed Workspace, Autodesk
Revit organization standards/templates, and one EDA peer. The addendum focuses
on organization-managed distribution, standards/profile selection as
configuration, portability, offline behavior, and the consequences for Q3, Q4,
and GP-C05. Peer conventions remain evidence rather than Datum doctrine.

<!-- OWNER-REQUIREMENT:GLOBAL-PREFERENCES:PROTOTYPE-FIRST-COURSE-CORRECTION -->
### Prototype-first course correction and resumed disposition sequence

The owner superseded the abstract-anchor process on 2026-08-26.
`docs/gui/prototypes/preferences-window.html` is now the working Preferences
design surface. The window is developed and reviewed as a real, integrated
product surface before remaining authority and interaction clauses are
extracted. Its current contents are clay evidence, not a specification.

`docs/gui/prototypes/preferences-ux-study.html` remains archival evidence for
the approved Q3/PX-V5 and Q4/PX-V6 dispositions and for the history of the
unselected Q5-Q10 ideas. It no longer drives new decisions.

The process boundary is explicit:

1. GP-C03 Q1-Q4 remain approved and are not reopened.
2. On 2026-08-27 the owner unpaused GP-C03 Q5-Q10 and resumed the
   one-question-at-a-time disposition sequence at Q5. Later that day the owner
   inserted two drawn questions immediately after Q5:
   Q5A establishes a new user's initial baseline, and Q11 governs the optional
   Start page. The owner approved Q5-A on 2026-08-27 and approved Q5A-B+C with
   the rendered replay amendment later that day. The Start-page boundary first
   opened as Q5B; after approving it, the owner renamed it Q11 and retained Q5B
   only as a historical alias. Existing archival Q6-Q10
   identities remain unchanged. Each
   remaining packet
   must use `preferences-window.html` as its primary reviewed visual evidence,
   cite the exact governed section anchors, and use
   `preferences-ux-study.html` only as archival structural evidence.
3. No remaining GP-C03 or GP-C05 clause may be authored ahead of what the real
   window draws. Codex extracts bounded clauses only after reviewing the
   applicable window anchors against Q1-Q4 and governing authority. A candidate
   that materially differs from the window cannot open an owner boundary until
   Claude renders it there first.
4. Every remaining packet must state the exact on-screen decision in one plain
   sentence, cite written evidence with exact file and line references, present
   genuine alternatives (or prove from ratified law why no alternative
   survives), and enumerate only the bounded clauses that approval establishes.
5. Prototype catalog entries, labels, defaults, classifications, and
   interactions remain draft until separately governed. Their presence does
   not create a descriptor, Project object, default, implementation obligation,
   dependency, or standards-conformance claim.
6. `first-run-study.html`, `guided-setup-study.html`, and
   `start-page-study.html` are Claude-owned clay visual truth. Q5A must compare
   the separate wizard (A), guided-in-place Preferences path (B), and optional
   assistant proposal path riding on B (C), including the minimum required
   baseline. Q11 decides whether the Start page exists, what truthful local
   state it may show, and its explicit no-news/no-marketing/no-alert-channel/
   no-network-load exclusions. Both are now dispositioned only by their recorded
   owner approvals; their placement and prototype registration granted nothing.
7. Q5A remains the single dispositioned sub-letter exception. The historical
   Q5B packet/evidence identifiers remain resolvable aliases for canonical Q11.
   Any newly inserted question uses the next free integer, beginning with Q12;
   no further sub-lettered question IDs may be created.

<!-- OWNER-REQUIREMENT:GLOBAL-PREFERENCES:V1-DESCRIPTOR-CATALOG -->
### V1 Preference Descriptor Catalog

The specification phase must deliver
`specs/GLOBAL_PREFERENCES_V1_DESCRIPTOR_CATALOG.md`: a complete initial catalog
of every subsystem's v1 descriptor, legacy source, classification, scope
eligibility, default, constraints, management and Project-seed eligibility,
apply behavior, accessibility metadata, portability, migration/retirement, and
implementation disposition. It must also classify apparent settings that are
actually Project authority, restartable state, transient state, or operation
input. This is governed specification evidence, never implementation-time
improvisation.

<!-- OWNER-REQUIREMENT:GLOBAL-PREFERENCES:SHARED-UNITS-ENGINE-CARRY -->
### Shared units engine

The Global Preferences specification must include the engine-owned shared units
contract in `GP_SHARED_UNITS_ENGINE_REQUIREMENT.md`: exact signed-integer
nanometer storage, measurement-system defaults, typed per-quantity overrides,
display-only precision including exact nanometers, and one unit-suffixed input
parser serving GUI, CLI, and MCP. The prototype supplies the user-facing design
evidence; the requirement preserves canonical storage and defines the shared
service seam without authorizing implementation.

### Adopted drafting-standard authority

Product Mechanics 035 records the owner's 2026-08-27 Candidate A disposition:
the documentation system owns the Project's adopted drafting-standard object.
Global Preferences may later define an explicit, receipted new-Project seed and
read-only inspection doorway, but it does not own or live-update the Project
object. Exact schema and operations remain the bounded follow-up
`dat-adopted-drafting-standard-object-er9`; this does not resume Q5-Q10 or
extract clauses from the unsettled prototype.

### Schematic drawing-theme authority

Product Mechanics 036 records the owner's 2026-08-27 Candidate B ruling. The
Global Preferences specification must carry one persisted machine Presentation
descriptor for **Schematic drawing theme**, selecting the complete governed
Dark or Light contrast system. Dark is the factory default; Light uses warm
paper `#E7E1D2`; users cannot edit theme members; dark chrome and Publish/print
behavior remain unchanged. Board Layer color scheme stays separate, and a Light
board theme is future governed work. The later owner directive resumes GP-C03
Q5-Q10 but does not expand or reopen this bounded ruling.

## Non-negotiable boundaries

- Global Preferences may seed a new Project; copied Project policy then belongs
  to that Project until an explicit governed change.
- Presentation preferences cannot hide authority from audit, discard records,
  weaken release gates, or change engineering meaning.
- Machine/user, organization-managed, Project, session, and contextual values
  require typed scope, precedence, provenance, validation, and refusal rules.
- The factory experience remains usable without configuration. Managed and
  regulated profiles add obligations without creating rival data models.
- Research must cover local/offline operation, atomic persistence and recovery,
  schema evolution, import/export, synchronization, accessibility, and safe
  handling of unknown or unavailable providers.
- No implementation, synchronization service, identity provider, dependency,
  or standards-conformance claim is authorized by this plan.

## Scheduled completion path

<!-- REQ:GLOBAL-PREFERENCES-SPEC:GP-C01 -->
### GP-C01 — Internal authority inventory

Inventory every current setting, preference, environment/configuration input,
Project policy, session-only option, GUI control, persistence path, default,
precedence rule, migration, and recovery behavior. Separate implemented facts,
contradictions, missing authority, and product-language assumptions with exact
code/spec/prototype evidence. The inventory must explicitly prove the current
presence or absence of managed visibility enforcement and first-release
onboarding/completion state.

<!-- REQ:GLOBAL-PREFERENCES-SPEC:GP-C01A -->
<!-- OWNER:GLOBAL-PREFERENCES-SPEC:GP-C01A:GP-C01A -->
### GP-C01A — Owner approval of the factual baseline

Present the committed inventory as a findings-first packet. The owner approves
or revises only the factual baseline before external research or architecture
decisions proceed.

<!-- EVIDENCE:GLOBAL-PREFERENCES-SPEC:GP-C01A-APPROVED -->
**Owner review — approved 2026-08-25.** The owner approved the committed
`GP-C01-BASELINE` in `GP_C01_INTERNAL_AUTHORITY_AUDIT.md` without revision.
This approval establishes the factual starting point for GP-C02; it does not
select a future authority model, adopt an external convention, authorize a
dependency, or authorize implementation.

<!-- REQ:GLOBAL-PREFERENCES-SPEC:GP-C02 -->
### GP-C02 — External and standards research

Research primary and authoritative sources for preference scope/precedence,
managed policy, local/offline persistence, schema evolution, migration,
recovery, import/export, synchronization, accessibility, and regulated design
configuration. Distinguish transferable mechanisms from product-specific
conventions and licensed-text-gated requirements.

<!-- REQ:GLOBAL-PREFERENCES-SPEC:GP-C02B -->
### GP-C02B — Domain-peer preference architecture addendum

Complete the owner-directed professional-domain comparison and extract bounded
requirements for organization control, standards-profile configuration,
portability, Q3/Q4 authority, and GP-C05 interaction. The addendum must precede
the Q3 owner boundary and may not adopt an external server as Datum authority.

<!-- REQ:GLOBAL-PREFERENCES-SPEC:GP-C03 -->
### GP-C03 — Typed authority and resolution contract

Define identities, value/schema types, scopes, precedence, provenance,
defaults, organization constraints, Project-policy seeding, contextual
resolution, validation/refusal, unknown-field behavior, and the query surface.
Q1-Q10 and Q11 are approved and remain closed. Q8-A/P2 establishes opaque
inactive preservation, explicit alias migration, deliberate non-deletion and
export-first removal, with one dedicated **Manage preferences** home. Claude
commit `70bf2a2` now renders Q9's full resolver-owned explanation beside the
selected row in the real Preferences window, including effective and retained
contributions, Q4 reasoning, descriptor facts, remaining user actions, and one
typed GUI/CLI/MCP answer. Q9-A establishes that one authoritative complete,
read-only typed result across all three surfaces. Claude commit `7089ee8` closes
the Q10 visual gap with paired visibility states, S1 per-user-profile, S2
machine-local, and S3 per-Project onboarding alternatives, a separate
managed-required-replay choice, and first-use/replayed guidance beside the
unchanged Release arm bar. Q10-A/S2/R1 establishes ordinary machine-local
one-and-done completion together with an independently eligible organization
teaching directive that may require dismissible, non-gating replay once per
Project within a sufficient user-granted AuthorityRelease. The combination
avoids making all Project users repeat guidance while preserving the deliberate
classroom/governed-shop teaching case. GP-C03 is complete. GP-C04 now reconciles
Claude commit `586eb0d` into a complete persistence, identity/synchronization,
Project-partition, migration, recovery, exchange, audit, typed refusal, and proof
contract. The owner-adopted GP-C05 refined Option A now extracts search-first
discovery, row-complete editing/provenance, resolver-owned explanation, settled
setup/Start-page/Revision carry-forwards, and accessible context states from
Claude commits `a061fca`, `e2127fa`, and `586eb0d`. GP-C08 is next. Complete
each only by
extracting and reconciling the reviewed Preferences window, never by drafting
ahead of it; a material visual difference returns to Claude before the owner
boundary opens.

<!-- REQ:GLOBAL-PREFERENCES-SPEC:GP-C04 -->
### GP-C04 — Storage, migration, exchange, and recovery contract

Specify canonical local persistence, atomic writes, crash recovery, schema and
value migration, downgrade/unknown-provider behavior, import/export,
synchronization boundaries, conflict treatment, audit evidence, and the exact
seam between preferences and governed Project policy.

<!-- REQ:GLOBAL-PREFERENCES-SPEC:GP-C05 -->
### GP-C05 — Interaction and visual contract

Extract accessible discovery, search, editing, effective-value/provenance
inspection, reset/override/refusal behavior, managed-state presentation, and
the Revision carry-forward surfaces from the owner-settled
`preferences-window.html`. After Q5A and Q11 are separately dispositioned,
reconcile their settled initial-setup and Start-page surfaces with that window
across unmanaged, managed, teaching, keyboard-only, narrow, and
assistive-technology states.
GP-C05 does not pre-author interaction clauses for the prototype to satisfy.
The completed contract is
`GP_C05_INTERACTION_AND_VISUAL_CONTRACT.md`.

<!-- HISTORICAL-ALIAS:GLOBAL-PREFERENCES-SPEC:GP-C05A:GP-C08 -->
<!-- REQ:GLOBAL-PREFERENCES-SPEC:GP-C08 -->
### GP-C08 — V1 Preference Descriptor Catalog

`GP-C05A` is the historical alias. No further sub-lettered step or question IDs
may be created; future additions use the next free integer.

Absorb the working prototype's sourced catalog as a clearly marked draft seed
list now. After the prototype and authority/persistence/interaction decisions
are stable, validate every candidate and publish the complete v1 catalog named
above. GP-C06 ratification is blocked until every subsystem is covered and every
entry is selected for v1, explicitly deferred, or classified as not a
preference. The draft seed cannot satisfy this deliverable by itself.

<!-- REQ:GLOBAL-PREFERENCES-SPEC:GP-C09 -->
### GP-C09 — Consolidated ratification packet

Produce the single findings-first, adversarially reviewable GP-C06 packet before
reopening the owner boundary. It must consolidate and trace every GP-C03 Q1–Q11
disposition, the GP-C04 storage/migration/recovery/exchange contract, the GP-C05
refined Option A visual and interaction contract, the GP-C08 descriptor catalog,
and the shared units-engine requirement. It must state explicitly that approval
authorizes no implementation or dependency and does not weaken Product Mechanics
034, 035, or 036. GP-C06 remains pending until this artifact is committed and
listed as completion evidence for GP-C09.

<!-- REQ:GLOBAL-PREFERENCES-SPEC:GP-C10 -->
### GP-C10 — Ratification correction and doctrine record

Close the revised GP-C06 boundary and correct the catalog, consolidated packet,
citations, count reconciliation, review limits, and newly reconciled Claude
visual evidence as one governed change. Restore `ProjectPolicySeed` and every
owner-forced phrase named by the revision; add explicit unit-seed precedence,
PM-034 non-blocking law, GP-C04/GP-C05/units clauses, and a numbered Product
Mechanics decision record. Exclude unsettled clay and adopted-standard
co-ownership claims pending bounded Claude reconciliation. Refresh the complete
workspace-documentation-and-revision digest in the same correction transaction,
never as a separate blessing. The same reconciliation must align the four
catalog-to-render stable-key anchors and provide real rendered rows for the seven
GP-C08 identities named by the owner audit; it must also retire the obsolete
vellum-as-print-toggle statement and register that documentation consumer on the
evidence route. GP-C06 may reopen only after GP-C10 is committed.

<!-- REQ:GLOBAL-PREFERENCES-SPEC:GP-C06 -->
<!-- OWNER:GLOBAL-PREFERENCES-SPEC:GP-C06:GP-C06 -->
### GP-C06 — Consolidated owner ratification

Present the committed GP-C09 adversarial packet with exact internal, external,
standards, authority, storage, migration, recovery, integration, accessibility,
and visual evidence. Record owner dispositions in governed doctrine and a
normative specification without authorizing implementation.

<!-- EVIDENCE:GLOBAL-PREFERENCES-SPEC:GP-C06-APPROVED -->
**Owner review — GP-C06 approved 2026-08-28.** The owner approved the corrected
consolidated packet exactly as presented. Product Mechanics 037 is ratified;
the packet's review exclusions and its no-implementation, no-dependency, and
PM-034/035/036 preservation clauses remain controlling. This approval advances
only to GP-C07 planning and grants no execution authority.

<!-- REQ:GLOBAL-PREFERENCES-SPEC:GP-C07 -->
### GP-C07 — Frontier placement and proof contract

Place bounded implementation, migration, conformance, recovery, accessibility,
real-project, and production-acceptance slices on the Frontier. Keep execution
behind a separate explicit owner authorization and Product Mechanics 029 for
any new dependency.

The governed decomposition is
`specs/GLOBAL_PREFERENCES_ENGINE_IMPLEMENTATION_PLAN.md`. It places the shared
Units service as its own cross-cutting prerequisite before Global Preferences,
then alternates a separate owner-decision gate before every execution slice.
The Preferences program proves typed resolution, exact unknown preservation,
portable/Project Capability refusal, copy-once receipted seeding, one
GUI/CLI/MCP explanation result, Claude-render screenshot conformance,
accessibility, real-project behavior, and production acceptance. GP-C06's
unreviewed agent authority, three clay rows, unspecified adopted-standard and
watermark seeds, and four zero-descriptor subsystems remain named blocked work.

The same planning transaction prepares the findings-first
`research/documentation-system/REV_I00_EXECUTION_AUTHORIZATION_PACKET.md`.
That packet uses PM-037 only as the now-complete Project-policy seam and asks
the owner to authorize at most REV-I01 technical integrity. It does not start
Revision implementation.

## Required outcome

Completion yields an owner-ratified Global Preferences authority and a bounded,
separately authorized implementation path. It must provide the exact policy
resolution seam required by Product Revision Engine `REV-I00` without starting
Revision, Publish, or Preferences implementation.
