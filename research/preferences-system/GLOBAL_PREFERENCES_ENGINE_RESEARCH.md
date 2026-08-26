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

<!-- OWNER-REQUIREMENT:GLOBAL-PREFERENCES:VISUAL-ANCHOR-LAW -->
### GP-C03 visual-anchor law

`docs/gui/prototypes/preferences-ux-study.html` drives GP-C03 decisions Q3
through Q10. Every packet must cite and reconcile its reviewed anchor:

- Q3 — `PX-V5` / `#px-v5`;
- Q4 — `PX-V6` / `#px-v6`;
- Q5 — `PX-V7` / `#px-v7`;
- Q6 — `PX-V8` / `#px-v8`;
- Q7 — `PX-V9` / `#px-v9`;
- Q8 — `PX-V10` / `#px-v10`;
- Q9 — `PX-V11` / `#px-v11`;
- Q10 — `PX-V12` / `#px-v12`.

If a packet proposes materially different presentation or interaction from its
anchor, Claude must render that candidate in the prototype before the owner
boundary opens. Prose may not outrun the visual source of truth.

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
Resolve owner questions one at a time against cited evidence.

<!-- REQ:GLOBAL-PREFERENCES-SPEC:GP-C04 -->
### GP-C04 — Storage, migration, exchange, and recovery contract

Specify canonical local persistence, atomic writes, crash recovery, schema and
value migration, downgrade/unknown-provider behavior, import/export,
synchronization boundaries, conflict treatment, audit evidence, and the exact
seam between preferences and governed Project policy.

<!-- REQ:GLOBAL-PREFERENCES-SPEC:GP-C05 -->
### GP-C05 — Interaction and visual contract

Define accessible discovery, search, editing, effective-value/provenance
inspection, reset/override/refusal behavior, managed-state presentation, and
the Revision carry-forward surfaces. It must visually resolve Managed Revision
Visibility and First-Release Onboarding across unmanaged, managed, teaching,
keyboard-only, narrow, and assistive-technology states. Where visual judgment
is material, issue a Claude-owned brief and reconcile the resulting
`docs/gui/prototypes/*.html` source of truth before disposition.

<!-- REQ:GLOBAL-PREFERENCES-SPEC:GP-C05A -->
### GP-C05A — V1 Preference Descriptor Catalog

After the authority, persistence, and interaction decisions are stable, publish
the complete v1 catalog named above. GP-C06 ratification is blocked until every
planned subsystem is covered and every entry is either selected for v1,
explicitly deferred, or classified as not a preference.

<!-- REQ:GLOBAL-PREFERENCES-SPEC:GP-C06 -->
<!-- OWNER:GLOBAL-PREFERENCES-SPEC:GP-C06:GP-C06 -->
### GP-C06 — Consolidated owner ratification

Present one adversarially reviewed packet with exact internal, external,
standards, authority, storage, migration, recovery, integration, accessibility,
and visual evidence. Record owner dispositions in governed doctrine and a
normative specification without authorizing implementation.

<!-- REQ:GLOBAL-PREFERENCES-SPEC:GP-C07 -->
### GP-C07 — Frontier placement and proof contract

Place bounded implementation, migration, conformance, recovery, accessibility,
real-project, and production-acceptance slices on the Frontier. Keep execution
behind a separate explicit owner authorization and Product Mechanics 029 for
any new dependency.

## Required outcome

Completion yields an owner-ratified Global Preferences authority and a bounded,
separately authorized implementation path. It must provide the exact policy
resolution seam required by Product Revision Engine `REV-I00` without starting
Revision, Publish, or Preferences implementation.
