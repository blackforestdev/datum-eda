# Checking Architecture — Design Rationale

> **Status**: Non-normative design rationale.
> The controlling checking specification is `specs/CHECKING_ARCHITECTURE_SPEC.md`.
> The controlling ERC specification is `specs/ERC_SPEC.md`.
> The controlling CLI/MCP surface is `specs/PROGRAM_SPEC.md` and `specs/MCP_API_SPEC.md`.
> This document provides design reasoning and domain context. It does not
> define contracts, command surfaces, or data structures.

## Purpose
Explains the design reasoning behind the separation of ERC and DRC,
the shared infrastructure decisions, and the relationship between
schematic-domain and board-domain checking.

---

## 1. Why ERC and DRC Are Separate

ERC and DRC answer fundamentally different questions:

| Aspect | ERC | DRC |
|--------|-----|-----|
| Question | "Is the schematic intent valid?" | "Is the board physically correct?" |
| Domain | Schematic | Board |
| Input | Schematic connectivity graph, pin types | Board geometry, board connectivity, rules |
| Knows about | Pin semantics, hierarchy, net driving | Clearances, widths, annular rings, copper |
| Location type | Sheet + position | Board layer + position |

They share severity levels, waiver concepts, and reporting shape (defined
in `specs/CHECKING_ARCHITECTURE_SPEC.md`), but their checking engines are
independent. An ERC implementation never reads board geometry. A DRC
implementation never reads pin electrical types.

---

## 2. Shared Infrastructure Rationale

### Severity
Three levels (Error, Warning, Info) are sufficient for EDA checking.
More granularity (e.g., "critical" vs. "error") adds complexity without
helping the designer make decisions. The three levels map cleanly to
CI/CD exit codes and to GUI marker colors.

### Waivers
The waiver model in `specs/CHECKING_ARCHITECTURE_SPEC.md` §4 uses
`CheckDomain` (ERC | DRC) and `WaiverTarget` (object-based or
rule+object-based). Key design decisions:

- **Domain-scoped**: An ERC waiver cannot suppress a DRC violation.
  This prevents accidental cross-domain suppression (e.g., waiving an
  ERC undriven-input that also masks a DRC unconnected-pad issue).
- **UUID-based targeting**: Waivers match by object UUID, not by name.
  This ensures waivers survive re-annotation (reference designator
  changes) as long as object identity is stable.
- **Visible when waived**: Waived findings still appear in reports
  (marked as waived). This ensures periodic review — a waiver from
  months ago may no longer be valid after design changes.

### Violation Identity
Violations need stable identity for waiver matching and cross-session
tracking. A violation is identified by its check type plus the sorted
set of involved object UUIDs. This means:
- The same violation on subsequent runs matches the same waiver
- New violations on modified objects are not accidentally waived
- GUI can maintain violation selection across re-runs

---

## 3. Cross-Domain Consistency

The controlling spec (`specs/CHECKING_ARCHITECTURE_SPEC.md` §5) states
that cross-domain checks are not part of ERC or DRC and are not a
dedicated third checking domain in the current contract. They belong
to synchronization/comparison subsystems.

The rationale: cross-domain consistency (e.g., "schematic says 6 pins on
VCC but board only connects 5") is better modeled as a forward/backward
annotation comparison than as a third checking engine. The ECO
(engineering change order) system in M4 already compares schematic and
board state — cross-domain "checks" are a subset of what ECO computes.

If cross-domain checks are later formalized as a checking surface, they
would extend the `CheckDomain` enum and the CLI/MCP surface through the
normal spec amendment process, not through docs/ additions.

---

## 4. Incremental Checking Rationale

After schematic operations, only ERC checks whose input data changed need
to re-run. After board operations, only DRC checks whose scope includes
modified objects need to re-run. Full re-check is always available and
is the authoritative result.

Incremental checking is an optimization for interactive responsiveness.
If incremental and full re-check ever produce different results, that is
a bug in incremental checking, not a feature.

---

## 5. Consumer Exposure Rationale

The M2 CLI and MCP surface is defined in `specs/PROGRAM_SPEC.md` and
`specs/MCP_API_SPEC.md`. The rationale for the current surface:

- `run_erc` and `run_drc` as separate tools (not a combined `check`)
  because they have different inputs and the designer may want to run
  one without the other
- `explain_violation` accepts domain context so it can route to the
  correct explainer (ERC explanations reference pin types and hierarchy;
  DRC explanations reference geometry and rules)
- JSON output format for CI/CD integration (structured reports that
  machines can parse)
- Text output format for human reading (default, human-readable summary)
