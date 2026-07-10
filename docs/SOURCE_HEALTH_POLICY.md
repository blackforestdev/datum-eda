# Source-Health Operational Policy

Status: active

This document is the operational companion to
`docs/decisions/PRODUCT_MECHANICS_022_SOURCE_HEALTH_GOVERNANCE.md`. Decision 022
is controlling. If this document, a manifest, a script, or a pull-request
instruction conflicts with it, the weaker instruction is invalid and the
conflict is a blocking governance defect.

## Contributor Workflow

Before changing source:

1. Run the source-health gate and note whether any intended write target is in
   the legacy debt ledger.
2. If a target is oversized, identify a cohesive ownership boundary that can be
   extracted without changing behavior.
3. Keep every new child below its applicable normal budget.

Before landing:

1. Run the source-health and policy-integrity regression tests.
2. Run the complete drift battery.
3. Record before/after line counts and structural evidence for every touched
   legacy file.
4. Ratchet its baseline and ceiling downward in the same change.
5. Remove a legacy entry in the same change when the file returns to its normal
   budget.

Registration is never the response to a new oversized file. Decompose it before
landing.

## What Counts as Structural Evidence

Qualifying evidence names the ownership moved, the real module boundary created,
the new destination files, and behavior-preserving tests. Good boundaries follow
domain responsibility, lifecycle, or stable type ownership.

The following do not qualify:

- `include!` or equivalent textual concatenation;
- import/re-export-only, forwarding-only, or registration-only shells;
- numbered or generically named continuation files;
- whitespace compression, statement joining, or deletion without ownership
  transfer;
- moving implementation into tests, fixtures, examples, or generated-looking
  authored files; or
- splitting only to place each child immediately at the maximum budget.

Thin roots are allowed when they remain readable module maps with visible
ownership boundaries. A Rust file is measured at 700 pre-test lines, with an
inline `#[cfg(test)]` tail measured separately at 350 lines; dedicated Rust test
files use 700 total physical lines. Recursive literal `include!` content is also
expanded and measured as one logical module with a 700-line limit, so physically
small trampolines cannot conceal a logical monolith. Other authored languages
use their physical-file limits from decision 022.

## Debt-Ledger Operations

For a legacy file:

- each value in `limits` is the last approved landed count and hard ceiling for
  that metric;
- live growth above any metric limit is a failure;
- live shrinkage requires that metric limit to ratchet exactly to the live count;
- A move or rename requires atomic ledger reconciliation and retains history.
- Reaching the normal budget requires immediate de-listing.

Ledger entries also state source kind, trigger commit/date, responsible owner,
target boundary, intended shards, lifecycle status, and evidence references.
Policy constants never belong in the ledger.

## Tripwire Response

When the gate reports a new oversized file, split it into cohesive modules; do
not add a debt entry.

When it reports growth in a known monolith, extract enough real ownership that
the original ends smaller than merge base, then ratchet its ledger values.

When it reports an oversized extracted child, redesign the boundary; moving the
same concentration into a different filename does not resolve the defect.

When it reports stale debt, reconcile the move/deletion or remove a now-compliant
entry in the same change.

When it reports policy weakening, restore the controlling contract. If a genuine
permanent policy change is intended, stop and use a later numbered Product
Mechanics amendment with project-owner approval.

## Temporary Emergency Exception

Do not create an exception merely to keep feature work moving. If an operational
emergency truly prevents immediate decomposition, prepare a later numbered
Product Mechanics amendment containing the exact temporary delta, reason,
expiry, retirement evidence, and affected tests. Obtain project-owner approval
through the protected CODEOWNERS review path. An expired or incomplete exception
is blocking.

## Review Checklist

Reviewers verify:

- discovery includes every authored source file added or moved;
- no new source exceeds its applicable budget;
- every touched legacy file is smaller than merge base;
- extraction owns cohesive implementation rather than textual/facade indirection;
- new children and test destinations remain healthy;
- ledger ceilings only move downward and compliant entries are removed;
- behavior tests cover structural movement;
- no policy, coverage, workflow, or required-check weakening is hidden in the
  change; and
- protected governance changes have project-owner approval.

## Required Enforcement Surface

The standard drift battery must invoke blocking checks for source discovery and
budgets (including Rust pre-test, inline-test-tail, dedicated-test, and recursive
literal-`include!` measures), touched-monolith comparison against merge base,
ledger schema and downward ratcheting, architectural-evasion detection, and
policy integrity. The
checker regression suite must exercise tracked, staged, untracked, renamed,
decomposed, regrown, test-moved, expired-exception, and policy-tampering cases.

The controlling decision, this policy, the checkers and tests, debt ledger,
standard drift wiring, CI workflow, and governance classifications must remain
under CODEOWNERS and protected-branch/ruleset review. In-repository checks are a
backstop; repository protection supplies the human authorization boundary.
