# Product Mechanics 043: owner-directed workflow revisions

Status: proposed amendment; exact independent review and owner-controlled
promotion are required before this mechanism becomes installed authority.
Issue: `dat-workflow-owner-revision-71j`.
Evidence route: `workflow-owner-revision`.

<!-- WDQ-043-AUTHORITY -->
## Decision proposed for review

Extend PM042 with an explicit owner-directed revision path for an external lane.
The absence of the new field preserves all existing behavior. Preserve PM025
selection, claims, tracker synchronization and completion validation; PM041/042
proof, independent review and promotion; PM022 source health; PM029 dependencies;
and the product/prototype owner's authority. No product acceptance follows.

Schema-2 `coverage` may contain `owner_revisions`, an array of closed
`{frontier_key, record_ref}` entries. Keys are unique and must classify an
existing external lane. `record_ref` is the existing governed `{path, marker}`
shape and must resolve uniquely. Both policy and records must already be in
externally selected authority; candidate-only grants refuse.

## Exact record and preserved history

`OwnerDirectedRevisionV1` contains exactly `schema_version: 1`,
`kind: "owner_directed_revision"`, `marker`, `frontier_key`, `issue_id`,
`disposition: "revise"`, `decision_ref`, `owner_instruction`,
`owner_instruction_ref`, `provenance`, `source_commit`, `source_step_id`,
`correction_step_id`, `review_step_id`, and `target`.

`decision_ref` resolves to promoted numbered doctrine. `owner_instruction_ref`
resolves to a separate governed instruction record containing the verbatim
nonempty instruction exactly once. `provenance` has nonempty `source` and
`recorded_at` strings stating where and when the instruction was recorded.
It is not a fabricated signature. No parser infers acceptance from a natural
language instruction, a model's recommendation or an agent's role name.

`source_commit` is a full resolving ancestor of promoted authority. Its item
must be specified, owner-decision-authorized, unclaimed and selecting the named
pending owner decision. The exact `target` is a full Frontier item with the same
identity, ordering, canonical-selection flag, dependencies, unblocks, summary,
title and other non-revision fields. It retains every existing governing document
and includes the numbered decision, revision record and instruction record.
It starts specified, planning-authorized and unclaimed.

Preserve completion outcome, execution/presentation policies, delivery mapping,
successor policy and every existing step's requirements and owner packet.
Only the selected source decision changes to complete with the additional
`review` evidence pointing to its actual revision instruction. Completion of
that decision means **revision directed**, not **product accepted**.

Append a pending planning correction depending on the source decision and a
pending owner review depending on that correction. The latter has its complete
ordered owner packet. Additional future steps, if explicitly included in the
promoted target, stay pending without completion evidence and depend transitively
on the new review. Their presence grants no execution permission.

## Permitted lifecycle

The grant may be installed while leaving the entire source item unchanged.
This preserves the other agent's ownership and lets that agent apply the
governance transaction rather than having infrastructure assume its role.

The owning lane may then apply only the exact target, take its synchronized
PM025 planning claim and perform the bounded correction. It may complete that
planning step only with real evidence and return to specified/owner_decision,
unclaimed, selecting the still-pending review. All other target steps remain
exact. Completed correction evidence cannot be discarded or rewritten, nor may
the candidate restore a committed correction to its old source boundary.

The grant never allows completing the new review, selecting execution, landing
the product, changing requirements, removing historical proof, dropping governing
classification or adding source permissions. Those actions require their own
actual owner disposition and reviewed authority transition. Existing production
source-scope and affected-enrollment checks still run after revision validation.

Pin the instruction, revision record and numbered doctrine bytes in candidate
and authority. Preserve original gate behavior for every lane without a grant.
Malformed shapes, wrong identities, unknown fields, missing/duplicate records,
stale source references, non-ancestor sources, changed instructions or doctrine,
unclassified authorities, rewritten historical steps, missing review gates,
unauthorized execution and acceptance must fail closed.

## Preparation, integration and activation

Preparation records the already-provided owner instruction; it must not invent
a second product approval. Reviewers assess the exact proposed transition and
its limits. A matching instruction string does not authenticate who supplied it;
external owner selection remains required. This amendment does not weaken
policy equality, trusted gate-byte checks, hook installation or activation rules.

The workflow repair and its source permissions must be scheduled/classified and
independently reviewed through the infrastructure process before installation.
Register this record shape in spec parity and its docs in governance and the
owning route. Include both positive and negative entry-point regression evidence.
Promoting gate code is separate from applying a Preferences revision record.

For the current case, preserve `/tmp/datum-gp-cm05-revision`. Preferences
requirements, route reconciliation, correction claim, implementation, native
measurements and eventual product acceptance stay with that product's other
agent and the owner. Do not apply its blocked proposal from this workflow lane.

Dependency/licensing impact: none. No new third-party code or service.
