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
   The registry can support linear alphabetic, linear numeric, ECSS-style
   issue/revision, ISO 19650-oriented, and organization-custom schemes. Datum
   has **no implicit default scheme** before owner disposition.
2. Each revision-bearing Configuration Item has a scheme-bound revision
   namespace. An immutable baseline manifest is the primary identity of a
   complex product configuration. A product-level human label is optional and
   never replaces or rewrites member identities.
3. Successor-work collection records technical divergence and change scope but
   binds no engineering revision identity. A label is allocated or explicitly
   reserved only through a later designer/release operation governed by the
   selected scheme.
4. Revision origin, enabled minting events, phase-build identity, suitability
   status, and prototype-to-production transitions are explicit profile policy,
   not consequences inferred from editing or label spelling.

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
| `REVISION_SEQUENCING_AND_ORIGIN_RESEARCH.md`, lines 1–202 | Synthesizes scheme, origin, milestone, product-composition, and field-practice candidates and supplies six visual owner questions. | It ratifies nothing; its normative-strength claims remain bounded by REV-C02 and the audit below. |
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

### Source-strength audit of the sequencing synthesis

| Synthesis claim | Decision-packet treatment |
|---|---|
| Technical history and human-issued revision are different identities. | Strong Datum invariant from REV-C01/REV-C03; product examples corroborate it. Do not attribute exact wording to EIA-649C until licensed text is reviewed. |
| ASME uses a dash origin, skipped letters, and post-release `A`. | Candidate ASME-profile rendering only. The current public ASME page confirms the standard's scope, but exact sequence clauses remain `Blocked-text` under REV-STD-DRAW-001..002. |
| ECSS uses issue/revision and new version after released modification. | Public ECSS-M-ST-40C Rev.1 is normative evidence for document issue/revision metadata, release integrity, and successor version after modification; exact profile mechanics still require clause mapping. |
| ISO 19650 uses separate revision and suitability/status axes. | Supported as guidance/profile evidence by UK BIM Framework examples and status guidance. It is not a universal Datum core scheme. |
| Fab spins, EVT/DVT/PVT builds, certification submissions, and organization stage gates act as milestones. | Hardware/PLM behavioral evidence. These become selectable owner candidates, not standards requirements. |
| No project-wide automatic roll-up should exist. | Supported by Datum's per-CI/baseline model and exact-composition evidence. Any parent/product revision is an explicit scoped identity, never inferred label propagation. |

“Industry consensus” in the synthesis therefore justifies presenting choices; it
does not bypass profile selection, licensed-text gates, or owner ratification.

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
  | IssueRevision
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
| Issue/revision | Two ordered components with explicit issue-increment and revision-reset rules. | `2.3` | No inference about which component advances without a typed minting event. |
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

A product/build-level label such as `EVT2`, `Prototype 2`, `Block 4`, or an
organization-controlled product revision may optionally identify the release
scope. It is a separate scheme-bound identity over the baseline. It does not
require member labels to match, and it never becomes the primary composition
key. A phase is not a build and a build is not a board spin; their typed
relationships must remain explicit.

## Required concrete visual study

Claude owns the HTML prototype. The owner decision must not be requested until
one study renders all states below at realistic Datum title-block and shell
sizes.

### V1 — Scheme registry and lightweight-profile candidates

Render one policy-selection surface with equal candidates for numeric spin,
alphabetic drawing, issue/revision, ISO 19650-oriented, and organization-custom
schemes. Every candidate shows scheme name, example marked **EXAMPLE**,
namespace scope, initial allocation rule, next-token rule, exclusions/rollover,
and whether it is proposed for the lightweight profile. The page states “No
Datum default has been ratified.” ISO `STATUS` and `REVISION` are separate.

### V2 — Origin candidates and unreleased truth

Render the same pre-release drawing in two competing states:

1. **No identity until boundary:** `REVISION — UNALLOCATED` plus technical
   history available in details.
2. **Honest provisional identity:** visibly provisional token, separate from an
   issued revision and never visually confusable with released authority.

Also render the ASME-profile initial-release dash as an **unratified,
licensed-text-gated candidate**, not as a core meaning of dash or Datum default.

### V3 — Released title-block scheme projections

Render one controlled drawing under every scheme candidate at real title-block
size: numeric spin, alphabetic, issue/revision, ISO revision+status, and
organization custom. Each shows drawing/document identity, revision namespace,
issued revision, baseline identity, status where applicable, and release state.
Compact title blocks may shorten presentation but Inspector detail preserves the
full scheme and namespace.

### V4 — Complex product and phase-build identity

Render a release-candidate Inspector and Navigator showing one baseline with at
least four independently revised members: bare board, assembly/BOM, controlled
document, and firmware. The baseline is the primary composition heading. Render
an optional `EVT2` build identity as a separate release-scope label over that
manifest, not a shared child revision or automatic roll-up. Show phase `EVT`,
build `EVT2`, and board spin as distinct related identities.

### V5 — Successor work without identity

Render the first post-release working divergence as:

```text
SUCCESSOR WORK
Based on: BL-2026-08-24-01
Change: Draft CHG-...
Engineering revision: UNALLOCATED
No revision identity has been reserved or issued.
```

Render the later explicit allocation surface where the designer selects the
boundary crossed, affected namespace/scope, and scheme; then previews a proposed
label. Preview is not authority until a typed allocation or reservation event.

### V6 — Status axis and prototype-to-production transition

Render ISO-oriented revision and suitability status as independently changing
fields. Separately compare continuing one sequence into production with closing
a prototype namespace and beginning a production namespace. A reset must never
erase lineage or make two identities ambiguous.

### V7 — Accessibility and ambiguity proof

All states require word+glyph redundancy, non-color-only distinctions, keyboard
reachability, narrow-pane behavior, and full scheme/namespace identity in the
Inspector when compact chrome or title blocks show only display labels.

## Candidate owner disposition after visual review

The visual study must present these as six separate owner questions, one at a
time, each led by exact file-and-line authority:

1. Which scheme families ship in the v1 registry, and does the lightweight
   profile select a default? No default exists until this is answered.
2. Does human revision identity remain unallocated until a declared boundary,
   or may a profile create an honestly marked provisional identity?
3. Which typed boundary crossings may mint identities, and which are enabled in
   the lightweight profile: manufacturing release, phase build, share/delivery,
   review baseline, certification submission, or declared baseline?
4. Are phase-build names such as `EVT1` and `DVT2` first-class optional
   release-scope identities over a baseline manifest?
5. Is suitability/status available in the lightweight profile or only in
   profiles that require it? It always remains separate from revision identity.
6. Is prototype-to-production sequence transition supported, and if so is it
   explicit per organization/profile or enabled by default?

Across every answer, per-CI namespaces and baseline-manifest composition remain
the candidate structural invariant, and successor-work collection binds no
revision identity. The visual review must still test whether the owner accepts
those boundaries rather than treating this prose as ratification.

Q2 and Q3 remain blocked until that disposition is recorded.

<!-- EVIDENCE:PRODUCT-REVISION-SPEC:REV-C03-IDENTITY-PACKET -->
