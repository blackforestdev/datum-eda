# GBB-P05 — preserve and scope unresolved baseline evidence (planning only)

**Step:** GUI-PERFORMANCE-BASELINE / GBB-P05 (planning/qualification; the lane's FINAL step). **By:** dev-agent-01 (claim-holder; front resumed b69dcf34). **Date:** 2026-10-26. **Reviewer (adversarial):** audit-agent-01.
**Bound (specs/GUI_PERFORMANCE_BASELINE_BUDGET_PLAN.md, REQ:GUI-PERFORMANCE-BASELINE:GBB-P05, from :163):** "Preserve and scope unresolved baseline evidence (planning only)" — keep the eleven gaps scheduled without blocking functionality, each with its governed future-planning outcome; "Planning must identify exact missing evidence, method capability, finite resource/observation limits and decision value before requesting any future execution scope"; "This step authorizes no collection, rebuild, observer implementation, renderer optimization or CPU optimization"; "No automatic execution successor exists"; "Replacement budgets stay undecided"; "The baseline issue and GPU qualification remain open; canonical functionality selection is unchanged."
**LANE DISCIPLINE (qualification RESUMPTION; FINAL step — the lane LANDS on completion, NO owner gate after):** GBB-P01✓/P02✓/P03✓/P04✓(owner gate passed). work_purpose=qualification ⇒ no feature_delivery, no step reason. This step SCOPES the eleven residual gaps and authors **NO numbers** and authorizes **NO measurement/optimization**. Representative baseline + GPU qualification remain OPEN after P05; it does not close qualification.
**ETM PLACEMENT (the GBB-specific trap — honored):** GBB's governing plan AND the entire `docs/reviews/gui-performance/baseline-budget/` dir are ENUMERATED consumers of the `gui-performance-recovery` route (reviewed_digest). This scoping doc and all P05 evidence land as NEW files under the NON-enumerated `docs/reviews/gui-performance/baseline-budget/p05/` subdir; it **references** the plan's gap table and the P01–P04 artifacts, it **edits none of them**. `check_evidence_traceability.py` must stay 29/131 (verified at commit). If any enumerated file genuinely needed editing, that would be an in-commit digest reconciliation flagged to PM first — not done here.

## §A — Inputs (source-pinned, READ-ONLY / reference-not-edit — all ETM-enumerated)
- Bound: the GBB-P05 marker + the 11-gap table + the closing planning-only paragraph (GUI_PERFORMANCE_BASELINE_BUDGET_PLAN.md).
- Reused P01–P04 evidence at their RECORDED scopes (bound: "Reuse collection-protocol.md, pointer-report.md and the adjacent threshold audit at their recorded scopes"): `docs/reviews/gui-performance/baseline-budget/collection-protocol.md`, `pointer-report.md`, `threshold-audit.md`, `baseline-inventory.json`, and the P04 disposition `pointer-owner-disposition.json` (owner accepted ONLY the limited pointer assessment; replacement budgets undecided — PM051 GBB-P04).
- PM051 (GP-051-01..06) + the numerical-provenance procedure (AOP:330-367) — cite-only.
- NONE of the above is edited; this doc references them by path/scope.

## §B — PM051 + planning-only posture (stated before any gap)
- **No numbers authored.** P05 scopes METHOD and gaps; every budget/threshold stays **UNDECIDED / nonblocking** (GP-051-03), never defaulted. A "future planning outcome" is a method to specify/pin/reconcile — NOT a threshold.
- **No execution.** No collection, rebuild, observer implementation, renderer optimization or CPU optimization; no benchmark run; no new harness/dependency (PM029). No automatic execution successor — any future execution needs explicit owner approval of its concrete scope + a synchronized Frontier update.
- **Qualification stays OPEN.** No claim that a representative baseline exists, that GPU qualification is closed, or that canonical functionality selection changed. The P04 disposition is neither expanded nor weakened.
- **Reference-host ≠ all-hardware** (GP-051-05): the one reference host does not establish product policy (gap 10).

## §C — The eleven gaps: PRESERVED + scoped (none dropped, closed, or numericized)
Each gap is preserved (scheduled, nonblocking for functionality), carries its governed future-planning outcome, and is scoped along the four bound dimensions — **exact missing evidence / method capability / finite resource·observation limits / decision value** — identified *before* any future execution is requested. No gap is resolved, closed, or converted to a number.

**G1 Idle** — *outcome:* specify a valid duration/zero-work method and reuse credits.
- Missing evidence: a causal-schedule method covering the 60-second idle recipe (current causal schedule does not cover it; only historical recipes exist).
- Method capability: a zero-work/idle duration method whose validity is established (not the historical recipe reused blind).
- Finite limits: bounded idle-duration sampling; reuse existing idle credits rather than re-collect.
- Decision value: idle is the zero-work reference that other workloads subtract against — needed to interpret any later active-workload figure.

**G2 Pan** — *outcome:* pin accepted gesture and camera/output oracle.
- Missing evidence: a reconciled held-pan input/camera oracle (none exists).
- Method capability: a pinned accepted pan gesture + a camera/output oracle that validates the panned result.
- Finite limits: a bounded gesture set at the existing camera semantics' scope.
- Decision value: without a camera/output oracle a pan timing cannot be tied to a correct displayed result.

**G3 Zoom** — *outcome:* pin complete delivered-delta and output validation.
- Missing evidence: fractional wheel-delta coverage (the receipt lacks fractional deltas).
- Method capability: a complete delivered-delta capture + output validation at the existing anchor semantics.
- Finite limits: a bounded delivered-delta range.
- Decision value: fractional deltas are where anchor correctness is most fragile — needed before a zoom budget is meaningful.

**G4 Preferences scroll/controls** — *outcome:* reconcile actual control state, host and timing boundaries.
- Missing evidence: non-Main observer coverage (current observer is Main-only).
- Method capability: a method that reconciles the actual control state + host + timing boundaries beyond Main.
- Finite limits: scoped to the historical control-proof surface.
- Decision value: a Main-only observer cannot attribute control-interaction cost correctly.

**G5 Pane transitions** — *outcome:* pin pane/focus/camera/lifecycle checks.
- Missing evidence: complete per-pane state observation (current is incomplete).
- Method capability: pinned pane/focus/camera/lifecycle checks over the historical sequence.
- Finite limits: bounded to the existing transition sequence.
- Decision value: incomplete per-pane observation can hide which transition carries cost.

**G6 Native window lifecycle** — *outcome:* specify complete host/epoch and lifecycle timing correlation.
- Missing evidence: a host/epoch the causal mode accepts (causal mode rejects another host/epoch; only archived recipes exist).
- Method capability: complete host/epoch specification + lifecycle timing correlation.
- Finite limits: scoped to archived-recipe lifecycle events.
- Decision value: lifecycle timing cannot be correlated without an accepted host/epoch.

**G7 Mixed interaction** — *outcome:* specify input conservation and terminal-output checks.
- Missing evidence: a byte/output oracle (existing PTY recipe lacks it).
- Method capability: input-conservation + terminal-output checks over the PTY recipe.
- Finite limits: bounded to the existing PTY recipe scope.
- Decision value: without a byte/output oracle a mixed-interaction run cannot prove input was conserved.

**G8 Total instrumentation effects** — *outcome:* define a valid comparison OR **retain explicit uncertainty**.
- Missing evidence: a valid cross-mode comparison (the semantic observer is common to both timestamp and causal modes, so its effect cannot be differenced out).
- Method capability: either a valid comparison method, or — honestly — an explicitly retained uncertainty if none is defensible.
- Finite limits: bounded to the already-measured timestamp-mode CPU effects.
- Decision value: instrumentation effect bounds how precisely any figure can be stated; **explicit uncertainty is an acceptable honest outcome here, NOT a resolved number** (GP-051-02 variability honesty).

**G9 Displayed responsiveness** — *outcome:* identify a validated displayed-output method and scope.
- Missing evidence: continuous displayed-output evidence (exact endpoints are not continuous display evidence).
- Method capability: a validated displayed-output method (input→observed-display, consistent with PM051 "GPU execution is not input-to-observed-display responsiveness").
- Finite limits: scoped to a defined displayed-output observation window.
- Decision value: responsiveness budgets (incl. the native-workflow preview budget) ultimately need a displayed-output method, not endpoint timers.

**G10 Supported-hardware expectations** — *outcome:* obtain explicit supported-configuration expectations and rationale.
- Missing evidence: a product supported-configuration policy (one reference host does not establish policy).
- Method capability: an owner-provided supported-configuration expectation set + rationale (an OWNER input, not a measurement).
- Finite limits: n/a (policy, not collection).
- Decision value: no budget can be product-binding beyond the reference host without this — reference-host ≠ all-hardware (GP-051-05). **Owner item.**

**G11 Complete DRM/resource accounting** — *outcome:* coordinate with residual S4/S5 accounting without duplicating qualification.
- Missing evidence: full client/live/retiring DRM-resource coverage (unresolved).
- Method capability: coordination with the residual S4/S5 accounting (not a new duplicate campaign).
- Finite limits: bounded to what S4/S5 already scopes; no duplication.
- Decision value: resource accounting completeness gates any DRM/resource budget; duplicating qualification wastes effort.

## §D — Reuse + resumption discipline + no-shrink
- **P01–P04 evidence REUSED at recorded scopes, not re-derived:** collection-protocol.md, pointer-report.md, threshold-audit.md, baseline-inventory.json referenced at their recorded scopes; P02 measurements NOT re-run; P03 planning NOT re-opened.
- **P04 disposition INTACT (no-shrink):** the owner accepted ONLY the limited pointer assessment (pointer-owner-disposition.json); this step does not re-interpret, expand, or weaken it — no representative baseline is claimed, no replacement number ratified, no GPU-qualification closure, no measurement authorized. Historical results + all correctness obligations + adjacent limits remain intact.
- **No-shrink of the owner's requirement:** scoping a gap's method is not reducing the requirement; the honest "retain explicit uncertainty" on G8 is honest incompleteness, not a shrink. No gap is dropped, merged away, or declared resolved/closed; qualification stays open.

## §E — Future-execution prerequisites (named, not triggered)
Per the closing paragraph: no automatic execution successor exists. Any future execution of any gap's method requires (1) explicit owner approval of its concrete scope and (2) a synchronized Frontier update. Each gap above states its exact missing evidence + method capability + finite resource/observation limits + decision value precisely so such a request can be made with provenance — but this step requests none and authorizes none. G10 (supported-hardware policy) is an owner input; the scoped outcomes feed the GPI dispositions and residual S4/S5 accounting without duplicating qualification.

## §F — Constraints + findings + owner items
- **Constraints:** C-GBB-1 (every budget/threshold stays UNDECIDED/nonblocking; P05 authors no numbers); C-GBB-2 (no collection/rebuild/observer-impl/renderer-opt/CPU-opt; no benchmark; PM029 — no dependency); C-GBB-3 (all P05 evidence new-files-only under baseline-budget/p05/, no enumerated file edited, ETM stays 29/131); C-GBB-4 (P04 disposition intact; qualification stays open).
- **Findings (forward/owner):** F-GBB-1 (G8 total instrumentation effects may have NO valid cross-mode comparison — explicit uncertainty retained, honest incompleteness, not a resolved number); F-GBB-2 (G10 supported-hardware policy is an owner input the reference host cannot supply — reference-host ≠ all-hardware).
- **Owner items (named, not decided):** the supported-configuration policy + rationale (G10); approval of any future execution scope for any gap's method (concrete scope + synchronized Frontier update); making any replacement budget binding (stays undecided until evidence + separate owner approval, per GBB-P04 / GP-051-04). P05 settles none.
- **Cross-references (preserve, not amend):** the plan's gap table + P01–P04 artifacts (referenced at recorded scopes); PM051 + the numerical-provenance procedure. No governed/enumerated-doc edit.

## §G — Status
GBB-P05 gap-scoping COMPLETE (first full draft): all ELEVEN gaps (Idle/Pan/Zoom/Preferences/Pane/Native-lifecycle/Mixed/Total-instrumentation/Displayed-responsiveness/Supported-hardware/DRM-resource) PRESERVED and scoped along the four governed dimensions (exact missing evidence / method capability / finite limits / decision value), each carrying its governed future-planning outcome; none dropped, closed, or numericized. **PM051 + planning-only held:** no numbers authored, budgets stay UNDECIDED/nonblocking, no measurement/rebuild/observer/renderer/CPU optimization, no benchmark, no dependency; qualification + GPU baseline remain OPEN; the P04 limited-pointer disposition is intact (no-shrink). **ETM honored:** this is a NEW scoping doc under baseline-budget/p05/ that REFERENCES the enumerated plan + P01–P04 artifacts and edits none of them; ETM must stay 29/131 at commit (verified before landing). Four constraints + two findings; owner items (supported-hardware policy, future-execution approval, budget-binding) named, none decided. **Ready for audit-agent-01 adversarial verification (direct hand-off; findings via PM). This is the lane's FINAL step — on acceptance the GBB-P05 completion LANDS the lane (no park, no owner gate); audit's verification receipt is the P05 completion evidence.**
