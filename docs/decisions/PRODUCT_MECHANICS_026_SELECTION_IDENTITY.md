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
