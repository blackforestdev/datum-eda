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
code/spec/prototype evidence.

<!-- REQ:GLOBAL-PREFERENCES-SPEC:GP-C01A -->
<!-- OWNER:GLOBAL-PREFERENCES-SPEC:GP-C01A:GP-C01A -->
### GP-C01A — Owner approval of the factual baseline

Present the committed inventory as a findings-first packet. The owner approves
or revises only the factual baseline before external research or architecture
decisions proceed.

<!-- REQ:GLOBAL-PREFERENCES-SPEC:GP-C02 -->
### GP-C02 — External and standards research

Research primary and authoritative sources for preference scope/precedence,
managed policy, local/offline persistence, schema evolution, migration,
recovery, import/export, synchronization, accessibility, and regulated design
configuration. Distinguish transferable mechanisms from product-specific
conventions and licensed-text-gated requirements.

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
the Revision carry-forward surfaces. Where visual judgment is material, issue a
Claude-owned brief and reconcile the resulting `docs/gui/prototypes/*.html`
source of truth before disposition.

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
