# Workflow delivery: refusal matrix and worked transitions

Status: WDQ-C03 test oracle adopted at WDQ-C05, not executed gate-test results
Owning route: `workflow-delivery-quality`
Consumer: `specs/WORKFLOW_DELIVERY_GATE_CONTRACT.md`; ratified PM041

Each test starts from a complete valid, tiny fixture and mutates exactly the
stated input. The future dependency-free tests assert exit code, diagnostic code,
affected key/step/scenario/path and **no changes** to candidate files/Frontier.
This matrix is a contract for implementation, not evidence of an existing gate.

| ID / code | Mutation or transition | Required result |
| --- | --- | --- |
| N01 `WDQ-CONTRACT` | Enrolled item starts execution with absent contract, malformed JSON, duplicate object key or unknown field | Refuse ready/claim; distinguish parse location from missing reference |
| N02 `WDQ-IDENTITY` | Contract names another issue/key; duplicate scenario or consumer; unknown scenario reference | Refuse; no proof substitution by similarly titled task |
| N03 `WDQ-AUTHORITY` | Missing/duplicate clause anchor, stale governing route, unresolved mandatory owner question | Refuse ready; report owning route, not a command to bless its digest |
| N04 `WDQ-COVERAGE` | Empty consumer/scenario list, missing dimension, required case omitted or unjustified N/A | Refuse. A prose reason conflicting with authority requires review rejection; checker cannot infer its truth |
| N05 `WDQ-CONSUMER` | Enabled About key has no production handler, or test-only handler exists but production key does not | Refuse activation even when class is gui_local |
| N06 `WDQ-CONSUMER` | Pointer disabled but shortcut invokes; no-project context invokes project mutation; export registry drifts from runtime | Refuse activation; tests hit actual production entry paths |
| N07 `WDQ-RESULT` | Registered no-op emits “success” but expected panel/state is absent | Native assertion fails; refuse verify/accept, not fooled by registration |
| N08 `WDQ-RESULT` | Engine/CLI result used for required native scenario; screenshot without input/dispatch evidence | Refuse verify; retain narrow engine result as infrastructure evidence only |
| N09 `WDQ-ARTIFACT` | Artifact missing, altered hash, empty log, path escape, symlink escape or required evidence only untracked | Refuse; never fetch a replacement or execute a command from JSON |
| N10 `WDQ-STALE` | Relevant handler, numeric dependency, fixture, contract, consumer mapping or authority changes after proof | Refuse reuse and acceptance; report exact changed dependency |
| N11 `WDQ-STALE` | New/deleted file under input root, dirty source omitted, build receipt has another input digest | Refuse source identity even if source_commit is unchanged |
| N12 `WDQ-RESULT` | Any required result fail/blocked/unverified, missing required dimension assertion or duplicate result | Refuse verify/accept; no aggregate percentage threshold |
| N13 `WDQ-REVIEW` | Same implementation/proof/reviewer session, copied event replay, absent replay or revise/reject review | Refuse accept. A forged distinct identity is outside automated authenticity; owner checks provenance |
| N14 `WDQ-RECEIPT` | Owner name/timestamp only, wrong packet/review digest, candidate-only receipt or changed receipt | Refuse accept until explicit owner-controlled authority promotion |
| N15 `WDQ-DEFECT` | Unresolved blocking defect or unapproved nonblocking deferral | Refuse accept; closing the tracker item alone is not replay proof |
| N16 `WDQ-TRANSITION` | Complete acceptance directly from pending, skip verify, owner_decision with execution claim, select alternate step | Existing selector plus delivery check refuse; canonical output/selection unchanged |
| N17 `WDQ-POLICY` | Remove enrollment, shrink input roots, remove required scenario, weaken gate code or alter trusted-ref source | Refuse under trusted runner until exact owner-authorized policy promotion |
| N18 `WDQ-TRUST` | Missing/unresolved authority/base ref; candidate HEAD silently used as trusted base | Invocation error 2, never report enforcement pass |
| N19 `WDQ-INDEX` | Staged handler differs from tested worktree; unstaged receipt masks staged missing proof | Refuse staged transition using index bytes; leave other sessions' dirty files untouched |
| N20 `WDQ-ENVIRONMENT` | Change renderer/scale/input platform outside reviewed equivalence but reuse proof | Refuse relevant proof reuse; Linux evidence never accepts untested platforms |
| P01 | Valid readiness contract, all mandatory questions disposed, current lease and authorized step | Pass readiness only; no proof or product-acceptance assertion |
| P02 | Future control deliberately disabled, explanatory behavior and no-invocation scenarios pass | Pass bounded unavailable-control activation contract; do not claim its future feature implemented |
| P03 | Supported Fit control uses real handler; pointer/keyboard, context and visible result pass | Pass native verification; accept still waits on review and owner receipt |
| P04 | Infrastructure operation has named consuming workflow, exact API proof and justified GUI N/A | Complete infrastructure checkpoint; no product acceptance or enabled-control permission |
| P05 | Unrelated document/subsystem edit outside reviewed dependency closure | Existing proof remains current; unrelated HEAD movement alone does not invalidate |
| P06 | Entire product proof passes, distinct independent replay approves, no blocking defects, trusted receipt binds hashes | Permit recorded acceptance transition only; no next-task selection, auto-close or successor authorization |
| P07 | Unenrolled legacy item/history unchanged; another session has unstaged Preferences edits | No new WDQ obligation or mass reclassification; existing unrelated gates retain their own verdicts |

## Worked audited case

Baseline `19d0758` contains actual C01 evidence from runtime source `6fa5aa3`:
Help/About appears enabled, activating it produces a refusal. Under the proposal
that is N05/N07, not workflow delivery, despite passing document/engine gates.
Correct pilot behavior can be honest unavailability with explanation; building a
new About dialog is not a prerequisite to testing the readiness mechanism.

Fit is the positive native control: real input gives a visible fitted view.
P03 additionally requires a fresh pointer/keyboard/context proof on the final
pilot bytes. C01 observations are seed evidence and cannot pre-accept that build.

## Review sensitivity and acceptance limits

Test N04 has two layers: machine checks missing disposition/reference/coverage;
independent review checks whether a claimed N/A is logically justified. N07
requires observable native assertions, not static introspection of intent.
N13/N14 require an owner-controlled trust boundary; no shared-worktree identity
string can authenticate a human. These are deliberate limits, not waived tests.

The adoption pilot must demonstrate every N/P case with hermetic fixtures and
the About/Fit production scenarios, recording actual results. If any test cannot
be enforced as specified, return the mechanism to its owner decision before
blocking rollout; do not silently downgrade it to a checklist.
