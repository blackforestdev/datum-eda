# Product Mechanics 044: incremental work and historical delivery

Status: owner-authorized correction candidate; installation remains pending.
Tracking: `dat-workflow-owner-revision-71j`, bounded maintenance associated with
OR-C01; this does not resume or complete the stopped seven-gap repair program.
Evidence route: `workflow-incremental-delivery`.

<!-- WDQ-INCREMENTAL-DELIVERY -->
## Decision and scope

The owner authorized the bounded correction and, separately, a one-time commit
hook bypass after direct checks. That exception authorizes preparing and
committing this correction, not changing installed hooks or trust. Exact
installation remains a separate owner-controlled action. Preferences code,
product acceptance, dependencies and Claude-owned prototypes are outside scope.

On installation, these rules supersede PM042's automatic source-change proof
escalation and PM041/042's application of completed historical proof to later
source revisions. Other authority, ownership and acceptance checks remain.

1. A source edit does not complete an activation or verification checkpoint.
   At structure/readiness, validate the contract, unresolved decisions and
   governing evidence, plus the existing source-permission and live-claim checks.
   Do not demand final delivery proof solely because implementation changed.
   Completing a proof checkpoint still requires current candidate-bound proof;
   review and acceptance still require their independent evidence and receipt.
2. An enrolled policy row may name `historical_ref`, a full ancestor commit
   containing the original completed delivery and its policy. This is an
   owner-promoted policy field, never a candidate-selected fallback to HEAD.
   Both the pinned and current item must remain landed, without authorization,
   claim or selected work. Issue/landing identity and completion steps, mapping
   and outcome must match. Reopening requires a separately authorized transition.
3. Validate archived delivery against that immutable Git revision, using the
   current validator and the archived contract, inputs, environment and receipt.
   Do not execute archived validator code. Do not chain archive pins. Report
   `historical:<phase>@<revision>`, not current acceptance. Explicit verification,
   review or acceptance requests continue to check the current candidate.
4. Historical artifacts remain recoverable through the pinned commit even when
   current files evolve. They are not claims about current code. Current source
   permission, review and acceptance obligations do not derive from an archive.

The initial candidate pins the completed pilot and rollout to installed authority
`a467ccc4dc87df54a6fa26926c26cd8d8ef4cf68`. The rollout's recorded structural
continuation remains structural; this change must not mislabel it as fresh proof.

The owner also directed unblocking GUI-launch candidate
`0eb46520fec663d6cd532a658bdad500ec8ded25`. Its permission delta adds only
`crates/gui-app/src/app_shell.rs`, `crates/gui-protocol/src/lib.rs` and
`crates/gui-protocol/src/kicad_board_materialization.rs` to the existing
GP-CM05E source scope. The owning agent must synchronize its claim before landing.
No product patch is applied here, and no GP-CM05E completion is asserted.

## Verification and installation boundary

Exercise historical source evolution, explicit-current stale-proof refusal,
unpromoted-pin refusal, terminal-state and completion immutability, original
evidence retrieval, CLI labeling and malformed-pin refusal. Preserve existing
trust, review, source-scope, environment and ownership tests. Check the real
candidate policy, selector and source permissions before requesting installation.

One maintenance commit is allowed to bypass its incompatible installed hook;
direct checks and the exception must be recorded in the commit body. Do not
disable enforcement globally or modify pinned support in place. This decision
does not authorize future hook bypasses or another governance overhaul.

## Candidate verification record

102 distinct focused tests passed across history (9), trust (20), source scopes
(17), readiness (9), CLI (10), coverage entry points (8), review checkpoints (9),
review-phase capture (1), coverage (10) and environments (9). Independent review
identified explicit-phase downgrading and current-environment coupling; both were
corrected with regressions and independently reinspected without remaining
blocking findings. These tests validate the workflow, not the Preferences product.

Source health, dependency authority, Cargo resource policy, specification
governance, evidence traceability, inventory parity, generated projection and
project-state invariants passed directly. Installed delivery enforcement is
intentionally not represented as passing against unpromoted gate bytes.
An in-memory check of the real policy confirms the three launch paths refuse
without the product agent's matching claim and pass with those exact additions.
No live product claim was modified. Final committed-candidate inspection and
installation remain separate from these preparation results.
