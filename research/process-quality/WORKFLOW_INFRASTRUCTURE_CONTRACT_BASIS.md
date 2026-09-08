# Infrastructure delivery contract: local evidence review

Task: WORKFLOW-DELIVERY-IMPLEMENTATION / WDQ-I02.
Issue: `dat-wdq-rollout-implementation-ffy`.
Owning route: `workflow-delivery-infrastructure`.

This is a review of Datum's existing implementation and approved rollout scope,
not external product research, mechanism ratification or a proof result.

The approved R03 packet requires the infrastructure lane to consume the same
readiness and separate independent-review discipline it proposes for product
lanes. The explicit mapping proposal names WDQ-READY, WDQ-I03 and WDQ-REVIEW;
I04 remains owner-controlled activation and I06 remains full rollout acceptance.

The actual consuming entry points are `main` in
`scripts/check_workflow_delivery.py` (staged and exact-candidate CLI modes),
and `selector_failures` in `scripts/workflow_delivery_selector.py` (called
by the real project-status selector). They read trust, snapshots and delivery
records and produce refusals; they do not implement EDA editing. Function-qualified
identities in this contract describe those Python entry points, not new Datum
tool-registry verbs or a claim that a production telemetry emitter exists.

The prepared candidate at `a5f3d219f8a4f43e3509cc1d89d38033313226ab`
adds coverage, source-scope, specification-clause and distinct-review checks
at these entry points. Its 223 passing workflow tests are producer-side
regression evidence, not independently replayed infrastructure delivery or
product evidence. Fixture factories can construct hostile repositories, but
their fabricated proof/event helpers cannot supply the actual delivery record.

Complete technical consumers of this source are the infrastructure specification,
its machine contract and the proposed PM042 amendment. Their shared reviewed
digest binds the real normative mechanism. Operational progress, proposed
checkpoint mapping, proof, independent review and owner receipts remain outside
this authority route. Otherwise appending the review result would change its
own authority hash. PM042 is also a consumer of the rollout-planning route;
changing it requires reconciling both routes. This separation does not permit
an implementation or progress note to override either normative contract.

The input list is an explicit candidate-source inventory, including production
entry-point imports and existing workflow test recipes. It is not a claim that
all candidate modules are installed in main. Before readiness, reconcile it
against the assembled candidate, actual fixture recipe, environment and all
producer dependencies. Excluding arbitrary caches is necessary; excluding a
real runtime input to preserve an old hash is forbidden.

This bounded contract establishes read-only validator behavior only. Real
owner-hook and controlled-promotion success, refusal and interruption proof
remain additional I03/REVIEW obligations. Three native product cohorts and the
Preferences lane retain their own authority and owners. No infrastructure test
answers missing CAD units, numeric entry, selection, snapping, connectivity or
native persistence specifications.
