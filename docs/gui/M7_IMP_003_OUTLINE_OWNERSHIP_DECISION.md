# M7-IMP-003 Outline Ownership Decision Memo

> **Ticket**: `M7-IMP-003`
> **Stage**: Stage 1
> **Track**: Imported board fidelity inside opening `M7`
> **Status**: Product decision made

## Purpose

Define the product decision required before implementing `M7-IMP-003`.

This memo exists because the unresolved question is not only how to parse more
geometry. The real question is what Datum will treat as authoritative board
outline ownership in the imported KiCad PCB review path.

## Current Problem

The current imported-board path only derives board outline from top-level
`Edge.Cuts` graphics.

Current evidence:
- [parser_helpers.rs](/home/bfadmin/Documents/datum-eda/crates/engine/src/import/kicad/parser_helpers.rs:466)
- [skeleton.rs](/home/bfadmin/Documents/datum-eda/crates/engine/src/import/kicad/skeleton.rs:618)

Observed behavior:
- if no supported top-level outline is found, the board skeleton falls back to
  the default `10mm x 10mm` box
- the canonical audited `datum-test` trick case embeds the effective outline in
  footprint graphics (`fp_line` / `fp_arc` on `Edge.Cuts`)
- that outline is currently dropped silently

This creates two distinct issues:
1. supported ownership rules are not explicit enough
2. unsupported ownership cases are not surfaced explicitly enough

## Decision

Chosen rule:
- **Option A: support footprint-embedded `Edge.Cuts` as imported board
  outline contributors for the accepted KiCad PCB subset**

Reason for the decision:
- this is a common authored practice in KiCad and other EDA packages
- the imported-board fidelity track should support that real-world ownership
  pattern rather than rejecting it as out-of-scope by default
- the canonical `datum-test` fixture depends on this rule

Scope note:
- this decision does not mean "guess outline from arbitrary footprint
  graphics"
- it means Datum explicitly supports footprint-embedded graphics on
  `Edge.Cuts` as outline contributors under a bounded imported-board rule

For implementation, Datum still needs explicit rules for:
- accepted primitive types
- transform handling
- composition behavior
- unsupported ambiguous cases

## Considered Options

Datum considered the following two ownership rules for imported board review.

### Option A: Support Footprint-Embedded `Edge.Cuts` As Imported Board Outline

Interpretation:
- treat footprint graphics on `Edge.Cuts` as valid authored board-outline
  contributors for imported-board review
- apply footprint transform semantics (`at`, rotation, layer placement where
  relevant) before composing the outline

Pros:
- the current `datum-test` trick case becomes importable as-authored
- the importer becomes more permissive toward non-canonical but intentionally
  authored board constructions
- review may better match some real-world boards that encode outline geometry
  through footprint machinery

Cons:
- ownership becomes broader and more complex
- importer must distinguish package-local graphics from actual board truth
  carefully
- this increases the risk of treating decorative or package-borne geometry as
  board authority
- support burden rises because transform and composition rules become part of
  the supported subset

Implication:
- `M7-IMP-003` becomes a bounded support-expansion ticket, not only an explicit
  rejection ticket

### Option B: Keep Board Outline Ownership Top-Level Only

Interpretation:
- only top-level board `Edge.Cuts` geometry is authoritative for imported board
  outline
- footprint-embedded `Edge.Cuts` geometry is not supported as board truth in
  the imported-board review slice

Pros:
- ownership rule stays narrow and easy to explain
- importer remains stricter about design-truth boundaries
- reduces the risk of accidentally promoting package-local geometry into board
  authority
- aligns better with the Stage 1 rule that unsupported cases should fail
  explicitly rather than be guessed

Cons:
- the current `datum-test` trick case remains unsupported
- some intentionally clever authored boards may require re-authoring or a
  clearer outline encoding before Datum can review them faithfully

Implication:
- `M7-IMP-003` becomes an explicit-bounding and diagnostic ticket

## Chosen Outcome

`M7-IMP-003` should now be implemented as a bounded support-expansion ticket:
- footprint-embedded `Edge.Cuts` graphics are part of the accepted imported
  board-outline subset
- silent fallback to the default placeholder board remains unacceptable
- unsupported ambiguous ownership cases must still fail explicitly

## Required Outcome Of The Decision

Regardless of which option is chosen, `M7-IMP-003` must ensure:
- supported ownership rules are explicit
- unsupported ownership cases do not silently fall through to a fake default
  board without clear diagnosis

That means the following behavior is no longer acceptable:
- silently dropping outline truth
- silently continuing with the default `10mm x 10mm` placeholder box while
  implying successful board review

## Option A Implementation Direction

Implementation contract must include:
- explicit support statement for footprint-embedded `Edge.Cuts`
- transform handling for footprint-local geometry
- tests proving transformed footprint outline contributes correctly
- clear rules for how multiple outline contributors compose

Minimum proof:
- the canonical `m7-datum-test-half-routed` fixture imports with the intended
  outline
- no unrelated footprint graphics are promoted accidentally

## Acceptance Criteria

This decision is ready to implement when:
- the chosen support rule is recorded in this document
- the follow-on implementation brief can be written without ambiguity

Current read:
- satisfied

## Next Step After Decision

Create the implementation brief for `M7-IMP-003` against Option A, then code
against the chosen ownership rule.
