# REV-C03 Revision Identity Decision Packet

> **Prerequisite decision packet — not ratified.** This packet resolves the
> revision-identity question exposed by the owner's REV-C03-Q2 revision. It does
> not answer Q2 or Q3, select a default sequence, allocate any revision, or
> authorize implementation. Visual disposition is required before approval.

<!-- REQ:PRODUCT-REVISION-SPEC:REV-C03-IDENTITY -->

## Decision being prepared

The owner is not being asked to choose `A` instead of `1`. The decision is
whether Datum adopts the following authority shape:

1. `RevisionScheme` is explicit profile/project policy selected from a registry.
   The registry supports linear alphabetic, linear numeric, ISO 19650-oriented,
   and organization-custom schemes. Datum has **no implicit default scheme**.
2. Each revision-bearing Configuration Item has a scheme-bound revision
   namespace. An immutable baseline manifest is the primary identity of a
   complex product configuration. A product-level human label is optional and
   never replaces or rewrites member identities.
3. Successor-work collection records technical divergence and change scope but
   binds no engineering revision identity. A label is allocated or explicitly
   reserved only through a later designer/release operation governed by the
   selected scheme.

## Why this decision must precede Q2 and Q3

The title-block research left its third owner question unanswered: “Default
revision scheme: ASME Y14.35 alpha vs numeric vs ISO 19650 status+rev?” The
REV-C03 working model then used `A`, `B`, and `01` as examples without first
ratifying either a scheme registry or a default. Those examples cannot decide
product authority.

Q2 asks how successor work begins. Q3 asks where engineering revision identity
is allocated. Both would be biased if “successor” silently meant “the next
alphabetic/numeric label.” Datum must decide what a label means, where it is
unique, and when it is allocated before either question can be answered.

## Evidence

### Internal authority

| Evidence | What it establishes | Limit |
|---|---|---|
| `TITLE_BLOCK_AND_DOC_CONTROL_RESEARCH.md`, lines 206–216 | Default revision scheme is an explicit unanswered owner question. | It supplies no owner disposition. |
| `PRODUCT_REVISION_ENGINE_AUTHORITY_MODEL.md`, design result and canonical vocabulary | Technical history is automatic; engineering issue is deliberate; technical and engineering revisions differ. | It does not ratify a label scheme. |
| `PRODUCT_REVISION_ENGINE_STANDARDS_MATRIX.md`, REV-STD-ID-001..004 | Controlled items, documents/issues, and package members require stable, resolvable identity. | The cited clauses do not establish one universal label sequence. |
| Same matrix, REV-STD-BL-001..005 | A baseline is an immutable configuration manifest with exact membership. | A baseline label is not a substitute for member revision identity. |
| Same matrix, REV-STD-DOC-001..003 and REV-STD-DRAW-001..002 | Document changes/issues must be controlled and traceable; detailed ASME drawing-revision practice remains licensed-text gated. | Public ASME scope cannot authorize a detailed alphabetic default. |

### External primary/product evidence

- [ASME Y14.35-2025](https://www.asme.org/codes-standards/find-codes-standards/revision-of-engineering-drawings-and-associated-documents)
  publicly states that it covers identifying and recording revisions. Its
  detailed practices are licensed; Datum therefore cannot claim an ASME
  alphabetic default without licensed-clause review and owner mapping.
- [SOLIDWORKS PDM 2026 revision-number documentation](https://help.solidworks.com/2026/English/EnterprisePDM/admin/c_revision_numbers.htm)
  separates automatic version numbers from assigned revision numbers and
  supports company-specific static/counter compositions and different schemes
  for different workflows or file types.
- [Autodesk Vault revision-scheme administration](https://help.autodesk.com/cloudhelp/Help/ENU/Vault/files/GUID-E38303DA-4DE5-4D70-99A7-EBDFB809B56D.htm)
  exposes alphabetic, numeric, multi-level, and copied/customized definitions
  assigned by category. This is behavioral evidence for a registry, not a Datum
  dependency or authority.
- [Autodesk Revit revision numbering](https://help.autodesk.com/cloudhelp/2024/ENU/Revit-DocumentPresent/files/GUID-4F962FA3-1998-4993-94DE-CB5BCD5F46A0.htm)
  supports numeric, alphanumeric/custom, and unnumbered revisions, with per-
  project or per-sheet numbering. This demonstrates that presentation sequence
  and scope are policy choices.
- [Autodesk Vault revision relationships](https://help.autodesk.com/view/VAULT/2026/ENU/?guid=GUID-1FAD3749-175C-485F-A09E-41EA5D41E5CA)
  records assembly relationships to specific component revisions. This supports
  exact composition membership rather than forcing every member to share one
  revision label.
- [UK BIM Framework ISO 19650 guidance, §2.6](https://ukbimframework.org/wp-content/uploads/2020/04/ISO19650GuidancePart4.pdf)
  describes status code as metadata communicating permitted use and workflow
  state. Datum must therefore represent ISO-oriented status separately from the
  information-container revision code; `S3-P02` may be a formatted projection,
  but not one conflated semantic counter.

The vendor products are behavioral references only. The standards sources
govern only when their exact edition, licensed text where necessary, profile,
and owner-approved mapping are active.

## Candidate authority model

```text
RevisionScheme {
  id, name, kind,
  token_policy,
  successor_policy,
  initial_allocation_policy,
  reserved_or_excluded_tokens[],
  formatting_policy,
  applicable_ci_kinds[],
  profile_ref,
  effective_interval
}

RevisionSchemeKind =
    LinearAlphabetic
  | LinearNumeric
  | Iso19650InformationContainer
  | OrganizationCustom

RevisionNamespace {
  id, scope, scheme_ref,
  uniqueness_policy,
  allocation_authority,
  history[]
}

RevisionIdentity {
  id, namespace_ref, scheme_ref,
  canonical_token,
  display_label,
  allocated_event,
  issued_by_release_id?
}
```

Registry entries are immutable while referenced by issued identities. A scheme
change creates a successor policy version and explicit migration/disposition;
it never reinterprets historical labels. A custom scheme must provide a finite
ordered token list or deterministic structured successor rule. Arbitrary free
text is a label, not a sequence.

### Candidate scheme semantics

| Registry kind | Canonical behavior | Example projection only | Required refusal |
|---|---|---|---|
| Linear alphabetic | Ordered configured symbols, exclusions, rollover, case, and initial token. | `A`, `B`, `C` | No “next” value when exclusions/rollover are undefined. |
| Linear numeric | Configured integer start, step, width, prefix, and suffix. | `01`, `02`, `03` | No allocation outside range or conflicting within namespace. |
| ISO 19650 information container | Separate status/suitability policy and revision-code policy; formatter may show both. | `Status S3` + `Revision P02` | Refuse a profile that conflates permitted-use status with revision identity. |
| Organization custom | Editioned ordered tokens or deterministic structured segments. | `EVT-QUAL-3` | Refuse ambiguous ordering, duplicate tokens, or mutation of an in-use scheme. |

The examples above are never defaults. A project/template/profile must select
and configure a scheme before an engineering revision identity can be allocated.

## Complex-product composition identity

The primary released product identity is an immutable `ConfigurationBaseline`
manifest. Its members retain their own stable CI identity and revision identity:

```text
ConfigurationBaseline BL-2026-08-24-01
  Board MAIN          -> namespace board:main       -> revision C
  Board IO            -> namespace board:io         -> revision 07
  Harness FRONT       -> namespace harness:front    -> revision P03
  Assembly drawing    -> namespace document:ASM-01  -> revision B
```

`C`, `07`, `P03`, and `B` have meaning only with their namespaces and schemes.
The baseline manifest—not a coincident label—proves which exact versions form
the released configuration.

A product-level label such as `Prototype 2`, `Block 4`, or an organization-
controlled product revision may optionally identify the release scope. It is a
separate scheme-bound identity over the baseline. It does not require member
labels to match, and it never becomes the primary composition key.

## Required concrete visual study

Claude owns the HTML prototype. The owner decision must not be requested until
one study renders all states below at realistic Datum title-block and shell
sizes.

### V1 — Registry candidates

Render one policy-selection surface with four equal candidates. Every candidate
must show scheme name, kind, example marked **EXAMPLE**, namespace scope, initial
allocation rule, next-token rule, exclusions/rollover, and the statement “No
Datum default.” The ISO-oriented candidate must render `STATUS` and `REVISION`
as separate fields.

### V2 — Title-block projections

Render the same controlled drawing under each registry kind:

```text
ALPHABETIC              NUMERIC
DRAWING  ASM-01         DRAWING  ASM-01
REVISION B              REVISION 02
BASELINE BL-...-01      BASELINE BL-...-01

ISO 19650-ORIENTED      ORGANIZATION CUSTOM
DRAWING  ASM-01         DRAWING  ASM-01
STATUS   S3             REVISION EVT-QUAL-3
REVISION P02            BASELINE BL-...-01
BASELINE BL-...-01
```

Labels must be rendered as realistic bound fields, never typed examples. A
working/unissued sheet must show `REVISION — UNALLOCATED`, not a guessed next
label.

### V3 — Complex-product UI

Render a release-candidate Inspector and Navigator showing one baseline with at
least four members using different namespaces/schemes. The baseline identity is
the primary heading. Each member row shows CI, current issued revision, scheme,
and exact resolved state. A separate optional product-label field is visibly
secondary and may be absent.

### V4 — Successor work without identity

Render the first post-release working divergence as:

```text
SUCCESSOR WORK
Based on: BL-2026-08-24-01
Change: Draft CHG-...
Engineering revision: UNALLOCATED
No revision identity has been reserved or issued.
```

Also render the later allocation/release preparation surface where the designer
selects the applicable namespace/scheme and previews a proposed label. The
preview must not become authority until the typed allocation/reservation event.

### V5 — Accessibility and ambiguity proof

All states require word+glyph redundancy, non-color-only distinctions, keyboard
reachability, narrow-pane behavior, and full scheme/namespace identity in the
Inspector when a compact title block shows only the display label.

## Candidate owner disposition after visual review

The later approval request will ask whether to ratify all three propositions as
one identity boundary:

1. explicit scheme registry with no Datum default;
2. per-CI namespaces plus primary baseline-manifest composition and optional
   product-level labels;
3. successor-work collection binds no revision identity; allocation is an
   explicit scheme-governed designer/release-time operation.

Q2 and Q3 remain blocked until that disposition is recorded.

<!-- EVIDENCE:PRODUCT-REVISION-SPEC:REV-C03-IDENTITY-PACKET -->
