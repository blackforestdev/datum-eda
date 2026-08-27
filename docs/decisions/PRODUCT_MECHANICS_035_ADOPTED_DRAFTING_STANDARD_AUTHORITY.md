# Product Mechanics 035: Adopted Drafting-Standard Authority

Status: ratified doctrine

## Context

Datum needs one Project-authority object for the drafting conventions used by
Publish composition: projection angle, dimensioning conventions, document
units, and sheet symbology. Those conventions are distinct from both the
standards registry that identifies their cited basis and the Product Revision
Engine that controls revision, status, release, and issued-document identity.

The Preferences window exposes this Project concern but does not make machine
preferences its authority. The owner reviewed the evidence and candidates in
`research/documentation-system/DRAFTING_STANDARD_OWNERSHIP_DECISION_PACKET.md`
and approved Candidate A on 2026-08-27.

## Decision

The documentation system owns one Project-authority adopted drafting-standard
object. `AdoptedDraftingStandard` is the current specification name; its final
schema name remains subject to the bounded follow-up specification.

The object:

- references exact `StandardsRegistry` identities, editions, and cited bases;
- governs projection angle, document dimension conventions and units, sheet
  symbology, and their relationship to Publish templates and annotations;
- is mutated and queried through typed Project documentation authority;
- consumes Product Revision Engine revision, status, release, and issued-record
  projections without redefining them; and
- may be initialized from an explicit, receipted new-Project seed selected by
  Global Preferences, after which the Project owns its adopted value.

The standards/compliance subsystem remains authority for standards identities,
applicability, claims, and evidence. It does not own Publish composition. The
Product Revision Engine retains the complete Product Mechanics 034 revision
coding and release boundary. Global Preferences may inspect or seed the adopted
object but cannot become its live authority or silently update an existing
Project from machine or organization preference state.

## Required follow-up

`dat-adopted-drafting-standard-object-er9` must specify the exact object schema,
typed operations and queries, StandardsRegistry references, template and
annotation behavior, migration, unavailable-standard behavior, provenance, and
new-Project seed receipt. The already-landed documentation-system specification
is not silently rewritten by this decision; the follow-up is separately tracked
and requires its own governance before implementation.

## Non-decisions

This decision does not:

- select a drafting standard, standards edition, projection angle, unit system,
  dimension style, symbol vocabulary, or factory default;
- ratify any unsettled Preferences prototype catalog entry;
- reopen Global Preferences GP-C03 Q1-Q4 or resume Q5-Q10;
- resolve the parked schematic paper-versus-dark-background question;
- authorize implementation, migration execution, or a dependency; or
- alter Product Mechanics 034 revision, status, release, or document-issue
  authority.

## Owner evidence

<!-- EVIDENCE:DRAFTING-STANDARD-AUTHORITY:OWNER-APPROVED -->

The owner replied exactly `DRAFTING-STANDARD-OWNER: approve Candidate A` on
2026-08-27 after receiving the evidence-backed candidate packet. This approval
selects ownership and subsystem seams only.

## Dependency and licensing impact

None. This decision grants no dependency or license exception under Product
Mechanics 029.
