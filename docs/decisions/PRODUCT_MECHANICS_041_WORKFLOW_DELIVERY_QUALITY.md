# Product Mechanics 041: workflow delivery quality

Status: **proposal — not ratified; no active enforcement**
Prepared: 2026-09-05; WDQ-C03/C04
Owner disposition: pending WDQ-C05 concrete-packet review

## Problem and decision proposed

The bounded WDQ-C01 native audit separates partial viewing from unavailable
manual authoring and demonstrates an enabled action with no working consumer.
WDQ-C02 maps foundational questions to their existing specification owners.
Together they show why engine tests, a prototype, a closed issue and a populated
specification cannot independently establish delivery of a manual workflow.

Adopt a small delivery contract attached to **existing Frontier completion
steps**. Require a reviewed manual scenario before implementation, actual
consumer behavior before activation, native workflow proof before verification,
and separate owner acceptance of the reviewed revision. Machines refuse missing,
inconsistent and stale evidence; they do not infer usability or approve work.

The exact proposed schema, validation algorithm, identity limits and integration
points are in `specs/WORKFLOW_DELIVERY_GATE_CONTRACT.md`. Its refusal matrix is
`research/process-quality/WORKFLOW_DELIVERY_REFUSAL_CASES.md`. The bounded pilot
and activation handoff are specified in the WDQ adoption packet. These are
governed proposals until owner approval and a numbered ratification transaction.

## Preserved authority

- PM025 / `docs/PROJECT_STATE_POLICY.md`: sole selection, lifecycle, claim and
  authorization authority. The new contract stores no second work status,
  assignee, task ordering or automatic next step.
- PM001/002/012: canonical mutations, manual-first delivery and quality baseline.
  Missing implementation cannot amend product intent. Open domain questions
  require their own scoped resolution; this decision supplies no CAD semantics.
- PM022: source budgets and no policy weakening; existing gates continue to run.
- PM029: no new third-party implementation or license exception.
- Existing evidence-route review and Claude's complete prototype file lane.
  A gate failure never licenses another lane to refresh authority or a golden.

## Scope and tradeoffs

Initial enforcement covers only the explicitly enrolled pilot transitions and
their required production paths. Existing landed work remains historical, not
newly certified or mass-reopened. Newly expanding the rollout requires a recorded
owner disposition and an explicit enrollment transaction; an agent cannot
declare an unreviewed feature exempt or enroll another session mid-edit.

Infrastructure may complete with a named consuming workflow and bounded proof;
it must not claim that workflow is delivered. Disabled future controls can remain
unavailable with honest explanation. An enabled control must have its actual
context-eligible production handler, not just an identifier in documentation.

Costs are one reviewed contract and focused replay evidence per enrolled slice,
plus revalidation after relevant changes. Evidence is reused only for unchanged
named inputs and scenarios. No requirement to rebuild because an unrelated
document or another subsystem's file changed. This deliberately favors fewer
demonstrably usable slices over large feature inventories with hidden gaps.

## Trust and owner boundary

The checker can verify hashes, exact references and structural separation. It
cannot prove a review was honest, authenticate an owner from a JSON name, or
resist an administrator replacing the entire trusted runner. Owner-controlled
approval invocations pin the reviewed packet and independent-review digests;
their durable receipt is checked against an externally selected authority base.
Editing a status, timestamp, route digest or environment variable is not owner
approval. A single shared worktree is a cooperation boundary, not cryptographic
separation between agents.

The ordinary implementation author cannot supply their own independent review.
A separate session/person must replay the required native scenarios, report
expected versus observed results and defects, and remain separate from the
implementation and original proof. The owner judges intent and interaction
quality using that evidence; proof preparation does not require the owner to
re-audit every code change.

## Adoption and ratification

WDQ-C05 approval accepts only the concrete planning packet. It does not mean
unimplemented checks ran, a native pilot passed, or new work is selected.
After approval, reconcile the numbered decision, exact contract, evidence route,
governance inventory and owner record in one transaction. Authorize the bounded
gate implementation through its own Frontier handoff. Enforcement remains off
until the specified pilot, negative tests, independent review and owner activation
decision pass. Revision of this proposal after approval requires fresh review.

The proposed 041 number was free when drafted; confirm availability at
ratification. Do not overwrite another session's independently ratified decision.
