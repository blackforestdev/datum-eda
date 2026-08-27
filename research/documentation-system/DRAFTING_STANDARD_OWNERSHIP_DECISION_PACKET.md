# Adopted Drafting-Standard Ownership Decision Packet

> **Status:** Owner decision required; architecture only.
>
> **Tracker:** `dat-drafting-standard-authority-jcz` (related to
> `dat-documentation-system-spec-y8z` and
> `dat-global-preferences-engine-qcv`).
>
> **Boundary:** No implementation, dependency, standards-conformance claim, or
> reopening of Product Mechanics 034 is authorized.

## 1. Exact decision

The Preferences prototype exposes a Project's adopted drafting standard:
projection angle, dimensioning conventions, document units, and sheet
symbology. It correctly separates that concern from Revision Engine revision
coding but deliberately leaves the owning object unratified
(`docs/gui/prototypes/preferences-window.html:121-124`).

The owner must select which subsystem owns the authoritative Project object.
This is not a choice of ISO versus ANSI, not a standard-edition ratification,
and not approval of the prototype's current label or options.

## 2. Existing authority boundaries

### 2.1 Publish/documentation owns composition

Publish annotations already include Dimensions, datum/tolerance symbols, and
other paper-scale documentation; `TitleBlockDefinition` owns graphics, fields,
images, page rules, and standards metadata
(`specs/PUBLISH_SPACE_SPEC.md:183-194`). The rendering book assigns projection,
units, and tolerance to fabrication-document composition
(`docs/gui/DATUM_RENDERING_BOOK.md:394-403`). Product Mechanics 020 explicitly
joins Sheets, Viewports, annotations, title blocks, and documentation control
(`docs/decisions/PRODUCT_MECHANICS_020_PAPER_SPACE_AND_VIEWPORTS.md:192-202`).

### 2.2 Standards authority supplies cited bases

Product Mechanics 010 defines a resolved `StandardsRegistry`, attached basis
references, and Project compliance metadata
(`docs/decisions/PRODUCT_MECHANICS_010_INDUSTRY_STANDARDS_COMPLIANCE.md:46-66`).
Users select and record Project standards and metadata through typed operations
(lines 68–104). The Standards specification separates semantics from visual
composition: a cited standard may define field meaning while the template
system owns placement (`specs/STANDARDS_COMPLIANCE_SPEC.md:536-560`).

The registry should therefore identify the adopted basis and edition; it need
not become the owner of sheet-composition behavior.

### 2.3 Revision Engine owns revision coding, not drafting composition

Product Mechanics 034 owns the human revision clock, EngineeringRevision
namespaces, revision-profile selection, and ISO revision/status separation
(`docs/decisions/PRODUCT_MECHANICS_034_PRODUCT_REVISION_ENGINE.md:30-58`). It
explicitly leaves Publish composition on the Publish side and supplies
revision/status/release projections (`:89-101`). PUBLISH likewise says those
title-block fields are Revision Engine projections rather than free composition
values (`specs/PUBLISH_SPACE_SPEC.md:190-194`).

### 2.4 Preferences may seed but cannot own Project law

The approved GP-C03 Q2 boundary keeps `ProjectPolicy` mutations on the governed
Project path. Global Preferences may later select a new-Project seed, but the
adopted Project object cannot remain a machine preference or live-following
organization value. The current prototype already presents document units and
drafting standard as Project-owned, shown in Preferences but edited in Project
authority (`preferences-window.html:121-124,148-162`).

## 3. Candidates

### Candidate A — documentation-system-owned adopted object (recommended)

The documentation-system specification owns one Project-authority
`AdoptedDraftingStandard` (final type name remains specification work). It:

- references exact `StandardsRegistry` identities/bases/editions;
- owns projection angle, document dimension conventions/units, sheet symbology,
  and their relationship to templates/annotations;
- is mutated through the Project's typed documentation authority;
- consumes PM-034 revision/status/release projections without redefining them;
- may be selected as a receipted new-Project seed by Global Preferences, after
  which the Project owns its copy.

**Consequence:** `dat-documentation-system-spec-y8z` receives a bounded follow-up
specification obligation for this object and its seams. The closed DOC-C01-C06
work is not rewritten silently; this is new tracked scope.

### Candidate B — Standards/compliance-owned Project profile

The Standards subsystem owns both the cited basis and drafting behavior.

**Benefit:** one apparent standards home. **Cost:** it burdens
`StandardsRegistry` with Publish composition and template/annotation rules. A
clean split between registry basis and a documentation-owned adopted object
converges back to Candidate A.

### Candidate C — Revision-engine-owned document profile

The Revision Engine owns revision coding plus drafting behavior.

**Cost:** this conflates two authority domains and contradicts PM-034's explicit
Publish boundary. Selecting it requires reopening Product Mechanics 034 and the
Product Revision Engine specification; it cannot be absorbed as follow-up prose.

### Candidate D — Global-Preferences-owned setting

The machine/user Preferences store owns the active drafting standard.

**Cost:** Project documents could change meaning with machine/provider state,
contradicting Q2 and the prototype's “shown here, edited there” boundary.
Preferences may own only a new-Project seed reference, not the adopted object.

## 4. Recommendation

Approve **Candidate A**. It follows all current authority boundaries:

```text
StandardsRegistry basis/edition
              ↓ referenced by
Project.AdoptedDraftingStandard  ← documentation-system authority
              ↓ governs
Publish annotations · dimensions · symbology · template behavior
              ↑ consumes projections from
PM-034 revision/status/release authority
```

Global Preferences remains outside that authority chain except for an explicit
new-Project seed selection and read-only inspection doorway.

## 5. Owner response

Reply exactly:

```text
DRAFTING-STANDARD-OWNER: approve Candidate A
```

or:

```text
DRAFTING-STANDARD-OWNER: revise — <required ownership correction>
```

Approval establishes ownership and seams only. Exact object naming, schema,
operations, migration, standard profiles, UX, and implementation remain the
tracked documentation-system follow-up.

<!-- EVIDENCE:DRAFTING-STANDARD-AUTHORITY:OWNER-PACKET -->
