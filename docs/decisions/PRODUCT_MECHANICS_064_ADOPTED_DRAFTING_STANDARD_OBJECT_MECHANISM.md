# Product Mechanics 064: Adopted Drafting-Standard Object Mechanism

Status: ratified design-level doctrine. This records the specified mechanism and
the owner-ratified standard/policy selections for the Product Mechanics 035
adopted drafting-standard object. It authorizes no implementation, no migration
execution, and no dependency. It asserts no standards conformance: conformance
claims are gated on the owner-side licensed-source prerequisite (F1), which
remains an external purchase under Product Mechanics 029.

## Context

Product Mechanics 035 ratified that the documentation system owns one
Project-authority adopted drafting-standard object and left the exact schema,
typed operations, standard/edition selections, unavailable-source behavior, and
migration model as a bounded follow-up (`dat-adopted-drafting-standard-object-er9`).
That follow-up completed its research (ADS-C01), full object-and-behavior
specification (ADS-C02), independent review (ADS-C03), and exact owner
ratification (ADS-C04, `EVIDENCE:ADS-C04-RATIFIED:2026-10-26`,
`docs/reviews/foundation-followup/adopted-drafting-standard/ads-c04-ratification-decision.json`).

This decision records the mechanism that specification fixed, at design level,
so the object has governing doctrine before any implementation is separately
authorized. It changes none of the authorities it consumes: the standards and
compliance subsystem (PM010 / `STANDARDS_COMPLIANCE_SPEC`) remains authority for
standards identities, applicability and claims; the Product Revision Engine
(PM034) retains revision, status, release and issued-record coding; Project
Working Units (PM040) remain the units authority; grid and snap (PM023) are
unchanged; Publish composition (`PUBLISH_SPACE_SPEC`) owns Sheet and template
bodies.

## Decision — the object and its schema

The documentation system owns one `AdoptedDraftingStandard` object per Project.
Its persisted schema name is `adopted_standard`.

- **Identity and ownership.** One object per Project, with a stable `ObjectId`, a
  guarded technical `ObjectRevision`, and a deterministic JSON shard that is
  byte-stable given identical authored data. After seed the Project owns the
  value (see Seed and receipt).
- **Authored adoption versus resolved presentation.** Authored fields are the
  adopted StandardsRegistry references and the projection, dimension, unit and
  symbology conventions the designer chose, plus their template and annotation
  relationships. Resolved presentation is what Publish composition derives from
  the authored adoption at render; it is never persisted as rival truth.
  Resolved presentation never mutates the authored adoption, and the authored
  adoption never carries a resolved pixel or scale value.
- **Invariants.** Exactly one adopted object per Project; every convention value
  carries an explicit basis reference or is explicitly unset or custom (no silent
  default); any change advances the `ObjectRevision` through the typed path,
  never silently. The schema is this specified shape, not an arbitrary schema
  inferred from existing settings rows (PM035): observed runtime state is
  evidence, not ratified schema.

## Decision — standards basis and the ratified selections

The object references exact `StandardsRegistry` identity, edition and cited
basis; it does not redefine registry identities. Each referenced basis stays
distinguishable under the `STANDARDS_COMPLIANCE_SPEC` six-disposition model
(declared, inferred, unknown, unsupported), never silently merged.

The owner ratified the following standard family at ADS-C04 (exact editions are
the ADS-C01 section C2 candidate identities, confirmed against the
StandardsRegistry at selection; licensed normative text is not redistributed):

- **Dimensioning:** ISO 129-1.
- **Lettering practice:** ISO 3098 — IBM Plex remains the product typeface per
  the ratified rendering program; the conformance posture follows the
  unavailable-source policy below.
- **Title-block fields:** ISO 7200 (field meaning, not visual frame layout).
- **Projection method:** ISO 5456, with **third-angle projection as the product
  default, explicitly indicated per drawing** (the electronics-industry norm,
  ISO-permitted when indicated).
- **Units:** SI, with millimetre presentation over the canonical integer-nanometre
  engine.
- **Symbols:** IEC 60617, already ratified (Fork B), carried unchanged.

Any section C2 edition ambiguity is resolved by the owner (routed to the
appointed manager), not by implementation convenience.

## Decision — unavailable or unlicensed source behavior

When a selected standard's normative source is unavailable, incompatible or
unlicensed, the affected fields and claims render **explicitly unknown**; nothing
is fabricated and no silent substitution occurs; the product never refuses to
save or work. A registry reference is neither redistributed normative text nor a
certification. **No standards-conformance claim is made without the licensed
source (F1).**

## Decision — typed authority

The object is mutated and queried only through engine-owned typed Project
documentation operations on the single guarded write path, shared by the manual
GUI, CLI, scripting and optional MCP/agent surfaces. There is no per-surface
writer (the shared-by-construction law) and no private writer for any Preferences
or Publish consumer; inspection cannot write, and the query surface is read-only
and revision-qualified.

Each mutation specifies preconditions; an expected-revision guard (optimistic
concurrency against the object's `ObjectRevision` and the Project model revision,
the same discipline as the engine write path and the Project writer lease);
whole-or-nothing atomicity; typed validation and refusal (invalid basis,
unset-required field, or conflict refuses with no partial write); a diff; one
authored transaction per committed change in the journal; and undo and redo as
append-only compensating transactions that advance the `ObjectRevision` — never a
history rewind (PM000D). Agent and MCP changes enter as proposals requiring
acceptance.

## Decision — units, conventions, templates and the revision seam

- **Units and conventions.** The adopted object owns projection angle, document
  dimension conventions and units, formatting, precision and rounding, and sheet
  symbology; Publish composition resolves presentation from the adopted value.
  Canonical geometry stays signed-integer nanometres; Project Working Units
  remain PM040's authority; grid and snap remain PM023's. Changing an adopted
  dimension convention changes presentation only, never canonical geometry bytes,
  and introduces no editor-specific units engine.
- **Templates and annotations.** The adopted object governs title-block field
  meaning and standards metadata; visual frame layout belongs to the TitleBlock
  asset. Templates seed ordinary editable Publish objects rather than owning live
  Project law. A Viewport's projected scale is instance-local, not the sheet
  drafting convention. Publish annotations never drive Design geometry.
- **Revision seam.** The object consumes PM034 configuration, revision, status,
  release and issued-record projections without a competing lifecycle. A drafting
  convention change advances the object's `ObjectRevision`, never allocates an
  `EngineeringRevision` and never mutates issued or released bytes, which remain
  immutable.

## Decision — seed, receipt and migration (design only)

- **Seed and independence.** A new Project may be initialized from an explicit,
  receipted seed selection carrying the copied value and basis and a provenance
  receipt (what was seeded, from which Global Preferences selection, and when).
  An unavailable seed fails in a typed way with no silent fallback. After seed
  the Project owns the value; Global Preferences cannot live-follow machine or
  organization changes into an existing Project and is a seed-only, not a live,
  authority. This seam is specified here, not implemented in the Preferences
  lane, and reopens none of the PM035 GP-C03 preference questions.
- **Migration (design, not execution).** The design covers existing-Project
  discovery and compatibility, explicit user intent and migration authorization,
  old/absent/unknown-data handling, conflict and refusal, atomicity, rollback and
  recovery, provenance and deterministic reopen. An existing Project with no
  adopted object resolves to an explicit unset or unknown state; no historical
  adopted standard, edition or receipt is ever manufactured from machine
  defaults. Migration design and migration execution are separate authorization
  boundaries; execution is not authorized here.

## Conformance obligations (specified, not performed)

Any future implementation must prove, for ordinary manual use and every shared
entry surface (GUI, CLI, scripting, MCP): exact value and basis resolution with
declared, inferred, unknown and unsupported kept distinguishable; template and
annotation impact; units fidelity (presentation changes never alter canonical
nanometre geometry); typed refusal of invalid or unavailable input with no false
conformance; seed independence; migration and refusal behavior; deterministic
append-only compensating undo, recovery and reopen; issued-record immutability;
and standards-claim language that never presents a registry reference as a
certification. Where a Publish or documentation surface is depicted, visual
reconciliation against a Claude-owned prototype is a separate, audit-attested
prototype-owner action. Native proof is a future-grant obligation, not satisfied
by a JSON shape and not performed under planning authority.

## Scope and non-authorization

This decision is design-level. It authorizes no implementation, no migration
execution, and no new dependency, toolkit or third-party package (PM029). The
owner-side acquisition of licensed ISO/ASME normative text (F1) is an external
purchase that remains outstanding; it gates standards-conformance claims only,
not this object or its mechanism. Adopted drafting-standard capability and
Publish production acceptance remain separate, separately evidenced product
outcomes. This decision reopens no closed documentation-system task, resumes no
Product Revision Engine execution, and absorbs no Global Preferences ownership.
