# Repository health and drift history

> Detailed evidence relocated from `specs/PROGRESS.md`. The tracker remains the implementation-status entry point; the generated Active Frontier alone selects work. Dated history below does not grant execution or establish a fresh PASS.

## Current Repo Health

Current repo health status (2026-07-02):
- `cargo test --workspace` passes (2,006 tests).
- 358 Python MCP tests pass.
- The full drift-gate suite (`scripts/run_drift_gates.sh`) is green.

Current M7 delivery rule (2026-04-16):
- opening `M7` work may not advance on a "low resolution but technically
  implemented" basis
- user-facing slices must be intentionally triggerable, externally observable,
  and supported by the minimum interaction/render substrate they depend on
- missing prerequisite work in selection, hit-testing, focus/relatedness,
  visibility, or render-state consistency is not "scope creep"; it is
  prerequisite completion for the slice already being claimed
- working note:
  `docs/gui/M7_DELIVERY_GATES.md`

### Drift RCA + Prevention (2026-03-25)

Drift cause (root):
- Prior alignment checks were primarily static text-presence checks.
- They did not verify factual claims against live repo state (for example
  daemon method count, git/CI status, and current MCP method catalog parity).
- `specs/MCP_API_SPEC.md` mixed historical deferral wording with current write
  support, creating internal contradiction.

Corrections landed:
- Added `scripts/check_progress_coverage.py` to enforce:
  - `PLAN.md` progress-block coverage for active milestones (`M0`-`M4`, `R1`)
  - `specs/PROGRESS.md` section presence and infrastructure-fact parity
  - MCP current-method parity (`dispatch.rs` ↔ `tools_catalog.py` ↔
    `specs/MCP_API_SPEC.md` list/headings)
  - stale deferral text rejection for write/save support
- Wired `check_progress_coverage.py` into CI (`.github/workflows/alignment.yml`).

---

## Legacy Implementation Evidence

The following milestone tables preserve historical/current implementation
evidence. They are not the active North Star for the new scope. Use them to
ground factual current-state claims, then promote only substrate-relevant facts
into "Scope Integration / Substrate Readiness" above.

---
