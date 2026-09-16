# Owner-directed revision path: internal authority audit

Status: repair preparation, not installed authority or product acceptance.
Issue: `dat-workflow-owner-revision-71j`.
Owning route: `workflow-owner-revision`.

The owner requested correction of the Global Preferences acceptance specification,
then separately directed repair of the workflow path while retaining Preferences
implementation with its other agent. The exact instructions are retained in
`docs/reviews/workflow-owner-revision/owner-instructions.md`; neither instruction
accepts the product or waives review.

At source revision `68a3a54da094d2846651b71dfb2316996983cd80`, PM042 and
`workflow_delivery_categories.py` permit the external lane to close its final
owner decision but reject changed authorization, governing documents, completion
shape or selected substep. That is correct for unapproved scope expansion, but
also rejects the owner's explicit revision direction. A matching conversation
quote alone cannot override the installed owner-selected authority.

Existing mechanisms to reuse: PM025 completion/claim/tracker validation;
schema-2 owner-pinned coverage; governed Ref resolution; separate external trust;
production source scopes; independent review; and explicit owner promotion.
An external correction should not be reclassified as product acceptance, a
source-code permission or an exemption from another enrolled consumer's proof.

The proposed repair adds an optional owner-promoted revision record rather than
loosening the existing external-lane comparisons. It pins the actual instruction,
its provenance, the original owner boundary, exact revised plan, correction step
and subsequent review. The same parser/checks serve CLI, selector and hook paths.
Candidate-only records, edited receipts and arbitrary approval words refuse.

The source instruction is evidence of intent, not authentication. Externally
selected authority is still the trust boundary. Installing a new gate or a grant
remains separately reviewed promotion; it must not demand a newly invented
product-approval response to replace the already-recorded correction instruction.

Prepared enforcement permits only planning and a return to pending owner review.
Later execution, acceptance or another correction requires its own authorized
transition. Previous obligations and proof remain intact. A workflow session can
install the grant while leaving the product lane at its original boundary; the
product owner applies the actual correction and owns its synchronized claim.

Hermetic tests use synthetic instructions and isolated Git repositories. They
exercise real enforcement entry points, not Preferences behavior. Source-clone
preparation creates no live assignment, installed trust or independent review.
The complete Preferences packet at `/tmp/datum-gp-cm05-revision` remains untouched.
