# Product Mechanics 022: Source-Health Governance

Status: ratified doctrine

## Decision

Datum treats source-module size and architectural decomposition as permanent,
blocking health constraints. A file that exceeds its normal budget is governance
debt, not an accepted new baseline. Existing debt is frozen, must ratchet
downward when touched, and cannot be normalized by editing a ledger.

This decision is controlling. Operational documents, manifests, scripts, pull
request templates, and CI configuration may implement or strengthen it, but may
not weaken it. A permanent change to this decision requires a later numbered
Product Mechanics amendment, explicit project-owner approval through the
repository's protected review process, and corresponding machine-enforcement
changes in the same change set.

## Why This Is Required

The earlier source-size controls were retired after their hand-maintained caps
encouraged continuation files and paperwork rather than cohesive modules. Their
replacement restored visibility, but registration could make an oversized file
pass, allowed material drift and growth headroom, and did not enforce burn-down
when a monolith was touched. This allowed the exception ledger to behave as a
mutable observation rather than a health boundary.

Datum needs both protections: automated discovery that cannot miss newly split
children, and architectural rules that reject textual or facade-only splits.

## Normative Rules

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, and **MAY** are
normative.

### SH-001: Complete authored-source discovery

The source-health gate MUST inspect the union of Git-tracked paths, staged
paths, and source-like paths present in the local working tree. CI MUST inspect
the complete checkout. At minimum, coverage includes authored Rust, Python,
shell, JavaScript, TypeScript, TSX, and implementation-bearing HTML throughout
source, test, example, benchmark, build-tool, script, and server trees.

Generated, vendored, or immutable reference artifacts MAY be excluded only by
an explicit, reviewed exclusion containing provenance, reason, and owner. A
directory name alone MUST NOT create an implicit exclusion.

### SH-002: Normal budgets

The normal budgets are:

- Rust production modules: 700 pre-test lines (the content before an inline
  `#[cfg(test)]` test module).
- Dedicated Rust test and fixture modules: 700 total physical lines.
- Inline Rust `#[cfg(test)]` tails: 350 lines.
- Python, JavaScript, and TypeScript authored modules: 700 physical lines.
- Shell implementation files: 400 lines.
- HTML containing embedded implementation code: 700 lines.
- A Rust module's logical recursive literal-`include!` expansion: 700 lines.

The gate MAY use additional semantic measures, but it MUST NOT replace these
limits with a weaker measure. Both the physical file and logical recursive
literal-`include!` expansion limits apply; passing one does not excuse failure of
the other. Changing a normal budget upward is a doctrine change and requires the
amendment process below.

### SH-003: New debt is blocked

A new or newly oversized file MUST fail the gate. Adding it to a legacy ledger,
backlog, or exception list MUST NOT make it pass. New source must remain within
the normal budget or be decomposed before landing.

### SH-004: Existing debt is frozen without headroom

Every legacy oversized file MUST have an explicit baseline equal to its landed
line count and a ceiling equal to that baseline. Growth headroom, drift
tolerance, and silent baseline refresh are prohibited. The ledger is a debt
register, not authority to grow.

### SH-005: Baselines and ceilings are downward-only

The gate MUST compare a proposed change with its merge base. A baseline or
ceiling MUST NOT increase. When a legacy file shrinks, its baseline and ceiling
MUST ratchet to the new landed count in the same change. Once a file returns to
its normal budget, its legacy entry MUST be removed; later regrowth is new debt
and is blocked by SH-003.

### SH-006: Touching a monolith triggers burn-down

Any content change to a legacy oversized file MUST produce a net physical-line
reduction against merge base and MUST include evidence of structural extraction
in the same change. Updating metadata or compressing formatting does not create
burn-down credit.

Necessary extraction MAY accompany a feature change when the extraction is
separately reviewable and protected by behavior tests. A dedicated
decomposition change MUST remain behavior-preserving.

### SH-007: Decomposition creates real ownership

Burn-down MUST move cohesive behavior, types, helpers, or tests behind normal
language module boundaries. Textual `include!` concatenation, import/re-export
or forwarding shells, renamed continuation files, facade proliferation, and
compression-only line shaving do not count as architectural decomposition.

This rule specializes and does not replace the touched-monolith rules in
`specs/INTEGRATED_PROGRAM_SPEC.md`.

### SH-008: Children inherit governance automatically

Every extracted child MUST be discovered and checked immediately, including an
untracked local child. A compliant child needs no debt-ledger entry. An
oversized child fails under SH-003; decomposition cannot transfer debt into a
new file.

### SH-009: Test movement cannot hide debt

Moving production logic into inline tests, dedicated tests, fixtures, examples,
or generated-looking authored files MUST NOT satisfy burn-down. Each destination
remains subject to its applicable budget.

### SH-010: Blocking enforcement

Source-health and policy-integrity gates MUST return nonzero on violations and
MUST run in Datum's standard required drift check. A warning, PR checkbox, or
status message is not enforcement.

## Debt Ledger Contract

The governed debt ledger records facts and lifecycle state only. It MUST NOT
define thresholds, tolerances, growth percentages, valid-policy semantics, or
global exemptions.

Each legacy entry MUST identify:

- path and source kind;
- an exact `limits` map for every applicable debt metric (`pre_test_lines`,
  `file_lines`, `inline_test_lines`, and/or `expanded_lines`), each serving as
  both the landed baseline and hard ceiling;
- the trigger commit and date;
- a responsible owner;
- a cohesive target boundary and intended shards;
- lifecycle status; and
- evidence or issue references.

Permitted lifecycle states are `active` and `exception-active`. A stale, missing,
renamed, or now-compliant entry MUST fail until reconciled in the same change.

## Temporary Exception Process

Normal debt registration is not an exception. Only an operational emergency may
temporarily exceed a frozen ceiling. It requires all of the following in the
same change:

1. A later numbered Product Mechanics amendment.
2. Explicit project-owner approval through the protected review process.
3. The exact path, prior ceiling, temporary ceiling, and maximum delta.
4. A concrete reason that cannot be satisfied by immediate decomposition.
5. An expiry date and retirement issue/evidence reference.
6. Tests preserving the affected behavior and the governance gate.

An expired exception MUST fail. An exception MUST NOT authorize a new oversized
file, weaken repository coverage, legalize facade/textual decomposition, or
permanently raise a normal budget.

## Governance Integrity

In-repository checks MUST compare protected policy surfaces with merge base and
reject an unamended weakening, including:

- raising a normal budget, baseline, or ceiling;
- reducing discovery coverage or broadening exclusions;
- removing a required source-health gate;
- deleting, renaming, or reclassifying this decision; or
- modifying an existing amendment instead of adding a later amendment.

Repository checks cannot prove human authorization because code and checker can
be changed together. Therefore the project MUST also protect this decision, its
operational policy, source-health gates and tests, debt ledger, drift-gate
wiring, workflow, and governance classification with CODEOWNERS review and
branch/ruleset protection requiring project-owner approval, fresh approval after
new commits, the required source-health check, and no direct bypass push.

## Acceptance Criteria

Enforcement is complete only when regression tests prove at least:

- discovery of new tracked, staged, untracked, nested-tool, and Rust integration
  test files;
- failure of a new oversized file and one-line legacy growth;
- failure of an upward baseline/ceiling edit against merge base;
- required same-change ratcheting after shrinkage;
- acceptance of cohesive module extraction;
- enforcement of the 700-line recursive literal-`include!` expansion budget and
  rejection of forwarding/re-export, continuation, and test-dumping evasions;
- automatic governance of extracted children;
- required de-listing at the normal budget;
- failure of stale entries, unclassified exclusions, and expired exceptions;
- failure of unamended coverage, threshold, doctrine, or gate-wiring weakening.

## Consequences

Feature work that touches a monolith carries a small structural obligation. This
is intentional: frequently changed monoliths are the highest-risk debt. Existing
oversized modules can remain untouched without forcing unrelated scheduled
refactors, but they cannot grow, and their debt must decrease whenever work
enters them.
