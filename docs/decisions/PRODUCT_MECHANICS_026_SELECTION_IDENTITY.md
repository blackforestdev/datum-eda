# Product Mechanics 026: Selection Identity

Status: ratified doctrine

<!-- EVIDENCE:UVT-S5-SPEC:S5-C12-DECISION -->

## Decision

Datum's selection is a **typed, stable-identity subject** — one subject at a
time, drawn from a closed vocabulary, obeying one lifetime law, projecting
through one visual law, and claiming exactly the authority its delivery
boundary grants. This decision ratifies the complete S5 selection contract
(`docs/gui/DATUM_UNIVERSAL_VIEWPORT_TOOLING_SPEC.md` §2.2.13–§2.2.22, §4.4)
as the **Layer-2 shared-tooling capability** of the taxonomy governed by
decision 023: selection identity is built once and configured per editor,
never reimplemented per editor.

Three pillars are ratified as doctrine:

**1. The subject vocabulary (UVT §2.2.20).** The project-workspace selection
is exactly one subject from a closed typed vocabulary: `None`; **Object** (one
class-typed stable authored identity); **Compound** (enumerated authored
identities with optional focus); **Run** (derived physically-continuous
connectivity scope through a ratified origin class); **Global Net** and
**Bus** (derived semantic identities); and the non-authored **Proposal**,
**Review**, and **Diagnostic** subjects (explicit-only acquisition,
lifecycle-bound). Enumerated subjects drop what stops resolving; derived
subjects re-derive membership per revision; both obey the §2.2.18 lifetime
law. The flat-string singleton `SelectionTarget` is the acknowledged
predecessor this vocabulary replaces in the S5A build.

**2. The same-identity / merely-related law (UVT §2.2.20).** Two pane
renderings project the same subject iff they resolve the identical subject
identity at the same model revision — *one identity, two cameras*. A relation
between different identities — *two identities, one relation* — projects
related context only: exact authored baseline, no accent, not counted. The
eight-mapping classification table in §2.2.20 is closed doctrine: net/bus
member projections are same-identity; component↔placed-symbol,
definition↔instance, bus↔member-nets, net↔parent-bodies, finding↔target, and
proposal↔produced-geometry are merely-related. **No build may reclassify a
mapping; adding or changing a mapping amends this decision.**

**3. The S5A/S5B delivery boundary and atomic refusal law (UVT §2.2.14).**
S5A ships acquisition, lifecycle, projection, and read-only inspection only —
move, rotate, mirror, lock/unlock, group, and editable fields are S5B-or-later
seams rendered visible-disabled-with-reason, never active, never hidden. Every
later operation over a selection preflights all members through one shared
batch guard: a locked, stale, incompatible, constrained, or invalid member
refuses the whole operation with an explained blocker report identical across
GUI, CLI, and MCP — no silent skip, no partial mutation, no implicit repair.

## Owner-approved D4 clarification — 2026-09-30

The owner approved preserving derived Run/Global Net/Bus subjects and explicitly
refusing membership-changing additive/subtractive authored acquisition,
including marquee, while preserving existing no-op and lifetime behavior.
UVT §2.2.2 and §2.2.20 record the exact interaction and proof contract. No
implicit Compound conversion, member exceptions, or capped-context membership
source is permitted. This resolves `dat-selection-derived-modifier-caa8`
through the existing selection route without changing the vocabulary,
parent/child granularity, same-identity mappings or revision law. The refusal
is consumer selection behavior; it introduces no engine mutation guard.
This owner approval is product/specification authority only, explicitly **not
S5A implementation execution authorization**. S5A-C02 remains pending.

## Owner-approved acquisition/membership clarification — S5A-IR-01, 2026-10-01

The owner explicitly selected: "Gate origin A; acquire the complete derived
subject including B, keep hidden B unrendered and disclose its membership."
This reconciles UVT §2.2.8/B-HL's electrical-expansion eligibility wording with
§2.2.13/§2.2.18/§2.2.20's complete derived identity through the owning selection
route. Eligibility applies to the acquisition origin/candidate; it never filters
Run/Global Net/Bus membership. Members hidden or class-filtered before acquisition
remain in the complete authoritative resolution. Hidden members emit no canvas
cue; a selection-class filter does not hide a still-rendered member, which keeps
its full selection treatment. Complete inspection inventory/summary discloses
membership without relying on the 256-ID transport list.

An ineligible origin cannot acquire a derived subject. Existing origination
classes, parent/child granularity, click ladder/explicit verbs, D4 refusal/no-op,
revision/lifetime, pane projection and non-mutation laws remain. `Ctrl+A` remains
the sole ordinary authored-acquisition eligibility bypass. No additional hidden
mutation authority, engine guard, subject kind or identity mapping is introduced.
Packet N05/N09 supplies explicit pre-acquisition hidden/filter oracles. This is
product/specification authority only; independent review of the revised packet
and S5A-C02 execution authorization remain pending.

## Owner-approved S5A native acceptance scope amendment — D2, 2026-09-30

This delivery's native acceptance is limited to **board and schematic**.
Unavailable native Footprint/Symbol Editor rows in UVT §2.2.16 remain explicitly
**unverified**, deferred to their respective editor work, not passed or deleted.
Footprint Editor proof is tracked by `dat-footprint-selection-native-6hfa`; Symbol Editor
proof by `dat-symbol-selection-native-euub`, both related to the existing editor-persona
design intake `dat-symbol-footprint-editor-design-5gs`. Re-entry requires the
normal native editor persona and its own explicitly authorized implementation/
proof packet. No reviewer availability or editor execution is asserted.

This numbered amendment changes the current native acceptance boundary only.
The complete specification/class matrix and shared owner/configuration seams
remain binding; board/schematic parent acquisition, all identity mappings,
read-only scope and future editor obligations remain unchanged. Pure profile
checks and placed-parent proof cannot pass unavailable native editor rows.
Dimensions and hierarchical sheets retain their separately ratified exclusions.
Board/schematic acceptance cannot be described as acceptance of all four editors.

## Owner-recorded S5A D1/D3 dispositions — 2026-09-30

Under PM051, S5A latency budgets remain **undecided**. Preserve deterministic
work, correctness and resource obligations. The packet's bounded diagnostic
observations are approved in scope but are not numerical acceptance criteria;
collection remains behind S5A-C02 execution authorization and independent review.
No adjacent resource/CPU requirement or separate performance S5 scope changes.

PM026's read-only precedence governs conformance §8: mutation batch-guard
R1–R4 are future S5B-or-later mutation proof. S5A proves selection-acquisition
refusal, disabled mutation seams, exact blocker disclosure and zero authored
mutation, without building mutation infrastructure or claiming R1–R4 passed.
D1–D4 specification reconciliation does not complete S5A-C02; execution remains
pending independent review of the revised pinned packet and exact authorization.

## What This Decision Does NOT Do

**It authorizes no implementation.** S5A execution remains separately
unauthorized; this decision changes the authority level of the reviewed
specification, not the build schedule. Two classes are **deliberate
exclusions from S5A** — not completed capabilities: board dimensions
(re-entry tracked by `dat-dimension-selection-reentry-kxk`, riding the
decision-020 documentation-system pass) and hierarchical-sheet selection
(re-entry tracked by `dat-sheet-interaction-reentry-9ee`, riding the
schematic-surface design pass, with double-click on a sheet body permanently
reserved for descend-into-sheet). The Application Status Bar remains
deferred and untouched.

## Why This Is Required

Selection identity is the substrate under every editor surface Datum will
build: P2.3 cross-probe consumes it directly, native authoring (schematic and
PCB tool contracts) mutates through it, and the compound Inspector, context
payloads, and AI collaboration all project from it. Left as spec prose, its
load-bearing distinctions — especially the mapping table — erode casually: a
one-line renderer change ("the cross-probed symbol would look nicer glowing")
silently breaks the identity model. As doctrine, such a change requires a
deliberate amendment. The contract earned ratification through the governed
S5 closure pipeline: the exhaustive identity/class matrix (S5-C01), fourteen
owner-resolved reconciliation choices (S5-C01A), the bounded-query, lifetime,
output, boundary, refusal, identity, and overlay contracts (S5-C02–C08),
owner-approved visual evidence (S5-C09,
`docs/gui/reference/selection-study.png`), the complete disposition ledger
(S5-C10, `docs/gui/DATUM_GUI_CONFORMANCE_SPEC.md` §8), and final owner review
with one revise round-trip (S5-C11, UVT §2.2.22).

## Relationship To Other Decisions

- **Builds on 023 (Universal Viewport Tooling):** S5 is the Layer-2 entry of
  the shared-tooling taxonomy; the S0–S4 backbone (grid, camera, hit,
  hover) is its landed substrate.
- **Builds on 021 (pane tiling) and 014/015/019/020:** cross-pane projection
  rides the pane model; visual law rides the design system and Rendering
  Book; paper-space viewports (020) own the dimension re-entry.
- **Constrains the S5A build and P2.3:** the cross-probe slice depends on
  completed S5A and reads under this decision's identity model.
- **Reaffirms the operation/commit model (000-series) and decision 017:**
  selection is consumer state, never journaled; all mutation flows through
  typed operations under the atomic refusal law.

## Evidence

Complete closure evidence is anchored in the governed corpus: UVT
§2.2.16–§2.2.22 (matrix, register with all fourteen `RESOLVED` entries,
contracts, review records), UVT §4.4 (overlay law),
`DATUM_GUI_CONFORMANCE_SPEC.md` §8 (42-assertion disposition ledger),
`docs/gui/reference/selection-study.png` + `docs/gui/reference/README.md`
(owner-approved visual reference), and the `dat-s5-selection-visual-contract-zid`
bead trail (fourteen OPEN resolutions, C02–C11 landings, the C11 revise
round-trip, and this ratification).

## PM053 C01–C03 owning reconciliation — 2026-10-01

[PM053](/docs/decisions/PRODUCT_MECHANICS_053_ELECTRICAL_IDENTITY_AUTHORITY.md) ratifies stable NetId final-state atomic nomination/fallback,
explicit logical-to-board identity relationships (pending/mismatch are lawful
incomplete states), and semantic Bus declarations with occurrence-qualified
bindings. These clauses control conflicting earlier summaries in this document:
read/cache/reopen never allocate identity; display rename preserves identity,
while a label edit changing electrical groups follows recorded transitions.
Bus drawing split/member reorder does not split its declaration; semantic split/
merge explicitly records survivor and complete redistribution. A declaration
can resolve with zero drawings. Full interface equality differs from related
subset/remapping; names/member equality never establishes Bus identity and Bus
has no board projection. Occurrence identity is source plus stable instance path.
Malformed references refuse atomically; pending/mismatch must never be published
as complete cross-domain Net selection. Authored identity/binding source is
distinct from recomputable graph/projection geometry. Canonical writes and replay
retain all recorded identities. No C04–C07, M1 numerical policy, geometry/CAM
support, selection mutation or native acceptance is ratified by this amendment.
