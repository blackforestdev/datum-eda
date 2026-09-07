# Workflow delivery pilot operational handoff

Recorded: 2026-09-06 UTC. Tracking: dat-workflow-gate-pilot-b3s.
Status: review reconciliation committed; renewed review acceptance required before G06P regeneration.
The previous G06 acceptance is historical for its exact review hash, not approval
of a corrected review. Activation remains off. G04 producer proof landed in `8d96e3a1` after the
production corrections in `dde90c75`; independent replay landed in `fd117641`.

<!-- EVIDENCE:WORKFLOW-DELIVERY-GATE-PILOT:WDQ-G06P-RECONCILED -->
## Corrected review and preparation diagnostics

Root preparation correction: `fbb04e52`. Independent-review correction:
`0aadec0192cf5e9742c3441f12e48c0742cc0af8`. The reviewer reinspected its retained
history and pointer-regression evidence, explicitly accounted for both closed
blocking defects with exact RESOLVED/REPLAY records, retained both nonblocking
deferrals, and cleared the owner-receipt pointer. Its original native replay,
producer proof and all 748 indexed proof/artifact paths remain byte-identical.
This was a correction to defect accounting, not a new native run or production fix.

The read-only committed-input command
`python3 scripts/workflow_delivery_prepare.py --check-review-only` first
reproduced both omitted IDs from the old Review and now passes for the corrected
Review, accounting for all four defects. It validates both Proofs, recorded
environments/correlations, the packet and exact defect references as preparation
diagnostics only. Its output explicitly reports owner_acceptance_checked=false,
trust_checked=false and promotion_performed=false. It is not a complete trusted
acceptance or independent-replay verdict. The actual enforcement validator and
its trust/receipt rules remain unchanged.

A real output-generation attempt from the clean corrected commit was also
refused with `record the exact owner acceptance before preparation`. The named
output directory was not created; no candidate, retention ref, hook or trust
configuration was produced. This intentionally verifies the remaining receipt
boundary instead of manufacturing a publishable candidate before acceptance.

All 121 workflow tests and 50 selector tests pass, including nine new checks for
omitted closed/replay defects, exact section-bound resolutions and replay hashes,
open blockers, unclassified references, deferrals, refusal before candidate-output
creation and no acceptance claims. Source health (1,802 files), traceability
(20 routes/114 artifacts), governance (204 classifications), project status
(50 items) and whitespace checks pass. The reserved reviewer independently
inspected this preflight and found no mismatch in the defect-accounting rules,
while explicitly retaining its limited preparation-only scope.

The coordinating claim is released at the renewed-review owner boundary below.
G06P is pending, not falsely completed: the old activation candidate is superseded
and a new publishable candidate cannot be generated until the corrected review
has an exact owner receipt. After that receipt, resume only G06P to regenerate
against a fresh clean committed base, then return to G06 for external promotion.
This ordering preserves one selected step and does not ask for promotion of an
unprepared candidate. Both closed fixes stay closed and both deferred Console
defects stay open. Preferences remains at GP-CM04; no GP-CM05 authorization.

<!-- REQ:WORKFLOW-DELIVERY-GATE-PILOT:WDQ-G06R -->
<!-- OWNER:WORKFLOW-DELIVERY-GATE-PILOT:WDQ-G06R:WDQ-G06R-REVIEW-PACKET -->
## Renewed acceptance of the corrected independent review

The exact revised findings are in `independent-review.md` and `review.json`.
The two additions are the already-fixed history-overlap and menu-pointer defects,
now explicitly resolved against the unchanged independent replay. The two prior
DEFER dispositions remain unchanged; no repeat deferral or new Console acceptance
is requested. The old ACCEPT remains immutable historical evidence, but its old
review hash does not approve this corrected record.

Packet: `ab71468a0acd0e845f5e7cd5e2b21aae5121a85dfd5c2d72fa2f5f372c5745a0`
Revised review: `9a90b1703fa2041c8ccd0d4584c8b856c1d64e120318da8d754716a998ba7443`

Required owner response (a request, not a recorded receipt):

```text
ACCEPT WORKFLOW-DELIVERY-GATE-PILOT/WDQ-G06 ab71468a0acd0e845f5e7cd5e2b21aae5121a85dfd5c2d72fa2f5f372c5745a0 9a90b1703fa2041c8ccd0d4584c8b856c1d64e120318da8d754716a998ba7443
```

WDQ-G06R sequences this required new-hash acceptance before regeneration; it does
not replace the ratified G06 acceptance identity or authorize live enforcement.
On the exact response, record a new uniquely referenced receipt without rewriting
the old one, complete only G06R, restore the G06P execution claim and regenerate
the candidate. G06 still owns actual owner-controlled promotion and verification.
No agent may substitute a new hash into the old owner response or activate the
gate on the strength of preparation diagnostics.

<!-- EVIDENCE:WORKFLOW-DELIVERY-GATE-PILOT:WDQ-G06P-RECONCILIATION-AUTHORIZED -->
## Owner-authorized correction after the activation refusal

Source: the owner supplied the exact promotion command and its WDQ-DEFECT
failure in this conversation, then replied `ok... proceed.` to the proposed
independent-review reconciliation. Recorded: 2026-09-07 UTC.

Candidate `9564368735dccada966388e76f7adcc2380b83b8` failed before publication:
`every proof defect requires review disposition`. Live HEAD remains
`c386a801dfc70577f232719a360a15c11f48ea8e`, the tree is clean, trust settings
are unset and the existing report-only hook remains installed. Do not rerun
that candidate's promotion block. Earlier preparation-success statements below
describe structural preparation only; they did not establish acceptance readiness.

The producer proof retains four defect IDs but the independent Review accounts
for only the two nonblocking deferrals. Its owning independent-review session
must assess and explicitly bind the already fixed Console history overlap and
menu pointer-authority defects to its existing replay. Do not delete defects
from the producer proof, weaken the validator, fabricate reviewer conclusions,
or carry the old ACCEPT forward to a changed review hash.

Reopen WDQ-G06P for this bounded correction and preparation regression checks.
The coordinating lane owns only preparation tooling/tests and operational
handoff/Frontier/projection/beads records. The reserved independent reviewer
owns review.json and independent-review.md; it may add exact replay-bound
resolution evidence there after checking its own retained evidence. No production,
prototype, authority-route, proof/replay bytes, dependency or Preferences edits.
The existing replay may be reused only if it actually proves both fixes and
all bound identities remain unchanged. G06 remains pending; corrected review
acceptance and external promotion still belong to the owner. A new publishable
candidate must wait for the corrected review's exact owner receipt.

<!-- EVIDENCE:WORKFLOW-DELIVERY-GATE-PILOT:WDQ-G06P-PREPARED -->
## G06P preparation validated; owner promotion remains separate

The owner-authorized sequencing correction and first-party tooling landed in
`2f9a696e`, `5c43f335`, `db82cf0c` and `9c924e38`. The first two real generation
attempts stopped safely on isolated projection/predecessor reconciliation errors;
their diagnostic directories were retained, not promoted. Corrected generation
succeeded from base `9c924e38ba3141ee22ac3cc7bf0f4ebff514deda` to detached proposal
`8830ce0b891b1275e20beb606475b107909225db`, with the complete output in
`/home/bfadmin/Documents/datum-wdq-preparation-owner-handoff/preparation.json`
and its adjacent `promotion.md`. This is intermediate preparation evidence,
not the final publishable candidate after this completion record lands.

The real run checked source Frontier/claim validity, the exact accepted receipt,
packet/review hashes and the entire 1,778-file live input closure. It generated
and committed exactly six proposal paths, passed spec governance, evidence
traceability and full structural Frontier validation, copied the external runner
and hook, and verified unchanged live HEAD, semantic index, worktree, local Git
configuration and input closure. Tracker closure exists only inside the detached
proposal; live tracker and main remain incomplete for G06. The only related
predecessor change removes this pilot from both remaining-unblocks projections.
No trust configuration, live hook installation or real enforcement invocation
using agent-selected authority was performed.

Verification: all 112 workflow tests and 50 selector tests passed; source health
(1,800 files), traceability (20 routes/114 artifacts), spec governance (203),
parity (16 inventories), progress coverage, Cargo resource policy, project status
(50 items), shell syntax and diff whitespace checks passed. The 14 preparation
tests include the real external validator through the proposed hook in a
disposable synthetic authority fixture: baseline passes, staged input drift
refuses with WDQ-INDEX/1, and runner drift refuses with WDQ-TRUST/2. Missing
trust, moving refs, internal runners, gate failure propagation, live-state drift,
proposal scoping and quoted command syntax are covered. These fixture checks
are not production promotion or a new native replay. No full drift-suite pass
is claimed.

The reserved independent reviewer rechecked the corrected implementation and
reported no remaining blocker in its read-only review. It verified the six-path
proposal and bounded predecessor reconciliation, the corrected sequencing and
the owner-only command ordering. It did not execute tests, edit files, promote
authority or change its original native-review verdict.

After committing this truthful G06P completion, regenerate the final candidate
from the fresh clean base using:

```bash
python3 scripts/workflow_delivery_prepare.py --output /home/bfadmin/Documents/datum-wdq-activation-final
```

The resulting `preparation.json` identifies the exact final candidate/base and
external runner; adjacent `promotion.md` contains the fully pinned owner-only
review, verification, fast-forward publication, configuration and post-install
commands. The generator refuses an existing output directory or a changed live
worktree. Do not publish an earlier candidate, force history, bypass a refusal,
or infer authority from a retained local candidate ref. Only the owner may
select and execute the final promotion. Preserve that external directory after
installation because the blocking hook uses its pinned validator.

Your exact ACCEPT and both DEFER responses remain valid and must not be requested
again for these unchanged hashes. G06P is complete; the parent remains open,
the claim is released, and G06 is the sole selected owner checkpoint. Preferences
stays at GP-CM04 without GP-CM05 authorization or resumption. Canonical task
selection, all bound authority/input digests and both open Console defects are
unchanged. No production, prototype, dependency or licensing change occurred.

<!-- EVIDENCE:WORKFLOW-DELIVERY-GATE-PILOT:WDQ-G06P-AUTHORIZED -->
## Owner-authorized preparation sequencing correction

The coordinating lane identified that G06 was demanding external promotion
before the candidate and runner were prepared. The owner was asked:
`May I correct the roadmap to authorize bounded preparation now, while preserving
your acceptance and keeping external promotion as a separate owner checkpoint?`
The owner replied exactly `I accept please proceed` in this conversation on
2026-09-06. This authorizes the operational preparation step below, not external
promotion, a new product acceptance, or a change to the ratified gate semantics.

<!-- REQ:WORKFLOW-DELIVERY-GATE-PILOT:WDQ-G06P -->
### Prepare the isolated activation transaction and owner-local runner

Insert WDQ-G06P as the sole execution step before the existing WDQ-G06 owner
checkpoint. Preserve G01–G05 evidence and the exact accepted contract, original
proof, independent review hash, owner receipt and deferrals. No bound authority
source is amended: this is the owner-authorized sequencing of already required
preparation, not a new acceptance mechanism or weakened trust check.

Prepare and test first-party local tooling and an isolated, reviewable proposed
activation commit containing only the approved enrollment, delivery mapping,
receipt references and final completion proposal. Keep the live main Frontier
and tracker incomplete until owner promotion and successful trusted verification.
The isolated proposal is not a second operating roadmap or a live completion
claim. Its local candidate reference, if needed to retain the object, must never
be used as an implicit authority reference. No feature branch or remote push.

Provide a pinned, external-to-the-development-worktree copy of the candidate's
validator and a local hook preserving the existing file-lane/staged-format gates.
Test missing-trust and candidate-drift refusal in disposable fixtures. Preparation
must not configure trust, install hooks, publish the proposal to main, or execute
an enforcement invocation using agent-selected authority. The owner must receive
the exact candidate/base IDs, runner path, review instructions and explicit
promotion/verification commands. Preserve Preferences at GP-CM04 and every
existing route/input digest. G06P may finish only with those preparation artifacts
and checks; G06 still owns actual promotion, activation and final closure.

<!-- EVIDENCE:WORKFLOW-DELIVERY-GATE-PILOT:WDQ-G06-OWNER-ACCEPTED -->
## G06 recorded owner acceptance

Source: project owner's explicit response in this conversation following the 84cfdd5f handoff.
Date: 2026-09-06 22:03:36 UTC (recording time).

The owner supplied the following response. The two deferral lines retain their
original display indentation here; the separately referenced disposition section
below normalizes that whitespace only, without changing either decision.

```text
ACCEPT WORKFLOW-DELIVERY-GATE-PILOT/WDQ-G06 ab71468a0acd0e845f5e7cd5e2b21aae5121a85dfd5c2d72fa2f5f372c5745a0 b4c214907e43eb4ae7328a005c33e7558f215791928a77cc469edb167b7d1fba
  DEFER dat-console-oversized-history-9uz
  DEFER dat-console-scroll-range-4y3
```

Both supplied digests were revalidated against the committed contract, original
proof, authority closure and independent review. Both Proofs, their identical
requested environments, disjoint correlated event records and the live
1,778-file source closure still validate. Adding only the owner-receipt pointer
does not change the accepted review hash or the independent verdict.

This accepts the finite native pilot and authorizes the previously proposed
owner-local activation preparation. It does not itself promote a Git revision,
install policy or configure approval authority. No authority/base/environment
configuration exists in this clone. G06 stays pending and the parent stays open
until the exact prepared transaction is externally promoted and scoped blocking
verification succeeds. Do not request this same ACCEPT response again unless
the packet or review changes. Preferences remains at GP-CM04; GP-CM05 is not
authorized or resumed, and canonical selection is unchanged.

<!-- EVIDENCE:WORKFLOW-DELIVERY-GATE-PILOT:WDQ-G06-DEFECT-DISPOSITIONS -->
## Explicit owner deferrals for this packet only

Source: the project owner's same explicit G06 acceptance response in this conversation.
Date: 2026-09-06 22:03:36 UTC (recording time).

DEFER dat-console-oversized-history-9uz
DEFER dat-console-scroll-range-4y3

These are owner-approved nonblocking deferrals for the exact finite pilot and
independent review above, not resolutions or general Console acceptance. Both
issues remain open for their owning lane. The oversized-record tail and blank
overscroll limitations remain real. This recorded disposition still needs
external authority promotion before the enforcement gate may trust it.

<!-- EVIDENCE:WORKFLOW-DELIVERY-GATE-PILOT:WDQ-G05-INDEPENDENT-REPLAY -->
## G05 completed handoff

The distinct reserved reviewer completed new native S01–S05 and pointer-regression
runs, inspected all 69 PNGs, passed 83 machine observations and recorded 19
required-dimension judgments. It read every source/consumer in both bound
authority routes, all 26 dimension N/A dispositions, all eight foundation
answers and the complete 1,778-file input closure. Its report, assessment, raw
replay, typed artifacts and `review.json` are committed in `fd117641`.

Independent disposition: approve **only the finite replay**. Both Console
findings remain nonblocking, open and not owner-deferred. Their prospective
owner-section references are unfulfilled; `owner_receipt` is null. Read
`independent-review.md` for actual observations, the extra failed diagnostic's
limits and the full triage. No independent verdict was authored by the producer.

The coordinating lane revalidated the committed original and replay Proofs,
explicit identical environments, builds, fixtures and input manifests, distinct
sessions, matching packet hash, and five plus five disjoint typed event records.
The review SHA-256 is
`b4c214907e43eb4ae7328a005c33e7558f215791928a77cc469edb167b7d1fba`;
the replay blob SHA-256 is
`b75b15e40e5f9188814ebaaa4b10b19c6b700c5d9b2cd2b3517fc2624a4c9c1b`.
Full trusted acceptance is deliberately unavailable without the owner receipt,
explicit deferrals and external promotion. Mechanical checks do not replace
the reviewer's visual judgments or the owner's acceptance.

The gate-mechanics evidence below and commit `2e164b5e` cover narrow infrastructure
completion, accepted-packet invalidation, unrelated-change preservation and
unchanged selection/claims. The clean-worktree precondition was resolved through
the owner's explicitly approved recoverable archive, not a filter or source
change. The unsigned policy and exact response below prepare G06 without
installing enrollment, accepting defects, setting trust or resuming Preferences.
The G05 claim is released at this handoff; the parent issue remains open for G06.

## G05 handoff boundary

The owner's subsequent `please proceed` starts G05 under the already approved
G03–G05 implementation/review window. The synchronized claim reserves the
existing independent lane's `independent-*` artifacts and `review.json`; the
original producer owns only coordination, gate-mechanics tests and unsigned
activation preparation. No new production correction is scheduled.

G04 closes only the bounded implementation/original-proof step. The separate
reviewer must replay all five mandatory scenarios on fresh identified bytes,
review all N/A dispositions and the complete input closure, triage the two
remaining Console findings, and prove the required acceptance-invalidation and
scoped activation mechanics before presenting G06 to the owner. Existing
read-only code-review work is not a substitute for that native replay.

The reserved independent reviewer session is `wdq-independent-review-20260906`;
the original producer remains `codex-wdq-g01-planning-20260906`. Do not copy
producer event artifacts into an independent replay. The current producer
assessment and typed packet are linked above/below, not an independent verdict.
Use the recorded commands in `proof-artifacts/environment.json` and the build
wrapper with new output directories; inspect the exact native captures and
source differences, not only the machine summary. Validate against a Git
revision/index: pre-existing ignored build-root artifacts remain deliberately
visible to the worktree freshness checker and were not deleted or filtered out.

Shared production files are committed and clean; this lane schedules no new
production edit in the handoff. Coordinate any future corrective edit explicitly
and repeat affected proof. Preferences remains paused at its own approval
boundary; this handoff grants neither GP-CM05 execution nor G06 activation.

## G05 gate-mechanics rehearsal (recorded before replay completion)

Recorded 2026-09-06 UTC against `200ea142`, with only this lane's G05 claim/test
changes. The reserved reviewer verified the actual GUI/CLI hashes and all 1,778
build-manifest members against the clean build snapshot and both the production
source revision and current committed input closure. It is collecting new native
observations; that identity check is not a native replay verdict.

Producer-side verification:

- `python3 -m unittest discover -s scripts -p 'test_workflow_delivery*.py' -v`:
  the existing 96 tests passed, covering the N01–N20/P01–P07 refusal matrix.
- `python3 -m unittest discover -s scripts -p 'test_project_status*.py' -v`:
  50 tests passed, including owner boundaries, exact presentation and claims.
- `python3 -m unittest discover -s scripts -p test_workflow_delivery_rehearsal.py -v`:
  two additional full-checkpoint rehearsals passed in disposable synthetic Git
  repositories. Infrastructure ends at verification with null activation and
  acceptance, no review receipt, no claim and no successor selection. An accepted
  synthetic product remains valid after an unrelated committed file/HEAD change;
  changed/new relevant source refuses with `WDQ-STALE`, and removal of its
  registered handler refuses earlier with `WDQ-ARTIFACT`. Restoring the exact
  source restores validity. Every check leaves candidate bytes, the receipt,
  Frontier, projection and tracker unchanged. These are not real owner receipts.
- The committed actual producer proof passed `validate_proof`, explicit
  `validate_environment` and `validate_correlations` for all five typed scenario
  event records. An in-memory delivery mapping from ready=G01, activate=G04,
  verify=G05, accept=G06 passed `validate_delivery(..., phase="verify")` against
  the same actual proof. No such mapping was installed in the live Frontier.
- Source health passed (1,796 source files); evidence traceability passed
  (20 routes, 114 artifacts). No route digest, prototype, golden or production
  source was changed for these checks.
- The combined suite passed again with 98 tests after adding the two rehearsals.
  Frontier/check-render, spec governance (203 classifications), parity
  (16 inventories), progress coverage and Cargo resource policy also passed.

Current original-proof identities, to be rechecked before an activation packet:

| Identity | SHA-256 |
| --- | --- |
| Contract | `21a57c4ffbc54828bd2a48ca2abd8fa470dcaa0d74cddf8fe38bcd7e2be10c30` |
| Canonical proof | `e1b83bb6e9e577516267c83ebdabf4129f2572e346cbf2e4384744ae3b2f597b` |
| Authority closure | `b598304e6cb13770cf024015eb4963584b0bc948c8280130d5104d812f15f562` |
| Packet | `ab71468a0acd0e845f5e7cd5e2b21aae5121a85dfd5c2d72fa2f5f372c5745a0` |

At this rehearsal checkpoint the independent review digest was not yet available;
the completed replay and unsigned request now appear above/below. No owner
receipt or defect disposition has been fabricated.
The local authority/base Git configuration remains unset. Hook, drift runner
and alignment CI still invoke report-only checks; no live enrollment exists.
G06 must use the owner-controlled promotion/runner boundary, not a candidate
HEAD fallback. The two open Console findings have now received independent
nonblocking triage and still require explicit owner deferral before acceptance.

### Live-worktree activation precondition

A read-only local clone at `2e164b5e`, checked out detached in
`/home/bfadmin/Documents/datum-wdq-activation-check-dSgXRJRt`, passes the actual
producer proof's full in-memory `verify` checkpoint from a **worktree** view:
all 1,778 declared inputs match. No policy, receipt, Git trust configuration or
live Frontier delivery mapping was installed in either checkout.

Before the owner-approved archive, the shared worktree had 1,816 files in that closure, including
38 pre-existing ignored runtime/test outputs. The same check refuses with
`WDQ-STALE` at `corrected-build/input-manifest.json`. The extra files are under
four text-native fixture `.datum` directories and the Console golden directory
(three `.actual.png`, `.diff.png`, `.report.txt` outputs). Runtime material
includes credential-bearing files: contents were not printed, committed or
reclassified as source. This is an actual activation precondition, not an excuse
to filter ignored inputs or claim that a clean Git status establishes freshness.

The owner answered `yes` to the explicit bounded-archive request in this thread.
After checking that none of the seven exact targets contained tracked files,
all four directory `lsof +D` checks and the three-output `lsof` check reported
no open handles; no Cargo/rustc process was active. The 35 files in the four
`.datum` directories and three Console outputs were moved, preserving their
relative paths, to the private mode-700 directory
`/home/bfadmin/Documents/datum-wdq-ignored-archive-xZsytO1s`. Nothing was deleted.
The before/after tar-stream SHA-256 was identical:
`2e4bd1911b4408b46f69218e29c5b56b73491d0818eb2484705ca6db651234eb`.
Credential-bearing content stays private and is not a repository artifact.

The seven recoverable targets are the `.datum` directories beneath
`text-density-repro`, `text-fidelity-repro`, `text-intent-repro` and
`text-transform-repro` in `crates/engine/testdata/golden/text/native`, plus
`routine-focused.actual.png`, `routine-focused.diff.png` and
`routine-focused.report.txt` in `crates/gui-render/testdata/golden/console`.
Restoration must check for new destination files first, then return only these
archived paths; never overwrite later work. Their restoration would deliberately
make this proof stale again until the input closure is reconciled.

The shared worktree now has exactly the recorded 1,778 inputs and passes the
same actual in-memory `verify` checkpoint. No tracked source, input-root filter,
policy, trust setting or other lane's development file changed. This resolves
the observed clean-worktree precondition, not future test pollution or external
trust promotion. Both development lanes remain in their original workspace;
Preferences resumption and G06 activation are still unauthorized.

<!-- OWNER:WORKFLOW-DELIVERY-GATE-PILOT:WDQ-G06:G06-ACTIVATION-PACKET -->
## G06 activation candidate reviewed by the owner

The owner's exact acceptance is recorded above. The following transaction was
reviewed with that response but remains uninstalled and externally unpromoted.
The native scope remains four action keys, nine existing entry surfaces and
five scenarios at Linux/X11 1280x768/scale 1.0. No Preferences, general Console,
new feature, other platform or native-authoring acceptance follows.

The proposed policy is `activation-policy.candidate.json` beside this handoff.
Its closed shape, legacy baseline and ratified decision reference validate.
It enrolls **only** WORKFLOW-DELIVERY-GATE-PILOT and preserves `200ea142` as
historical baseline. Its activation reference deliberately names a future owner
section, not an already approved enrollment. No live policy exists.

The proposed `completion.delivery` value on the existing item is:

```json
{
  "contract_path": "specs/workflow_delivery/pilot.contract.json",
  "checkpoints": {
    "ready": "WDQ-G01",
    "activate": "WDQ-G04",
    "verify": "WDQ-G05",
    "accept": "WDQ-G06"
  }
}
```

The checkpoint mapping has passed actual producer verification in memory; it
creates no new completion steps, ordering, claims or lifecycle. The final trusted
comparison base must be the committed **G05-complete** handoff, not the earlier
G04-only baseline: acceptance cannot skip the verified baseline.

Proposed activation is owner-local, through the PM041 owner-controlled runner
boundary. G06 first records the owner's exact response and explicit defect
dispositions, prepares the enrollment/receipt/completion transaction and its
scoped blocking integration, and validates existing gates. The owner then
reviews and externally promotes that exact commit. The implementing lane must
not choose its own trusted authority, set approval credentials or infer promotion
from an ACCEPT line. Only the trusted revision's validator may certify the final
candidate. Local/CI report-only output is not enforcement evidence.

Owner-controlled configuration must explicitly identify the full authority and
comparison-base commits plus the reviewed environment path. The trusted runner
must execute outside candidate-controlled code, against the candidate revision
or index, preserving the existing file-lane and staged-format checks. Relevant
edits, missing configuration, stale/altered evidence and candidate policy/gate
changes must refuse; unrelated edits must retain the proven behavior. No remote
CI activation or tamper-proof security claim is included. If that controlled
runner cannot be established, stay report-only and leave G06 incomplete.

Final verification invocation shape, with values supplied by the owner rather
than inferred by an implementing agent:

```text
python3 <owner-controlled-trusted-scripts>/check_workflow_delivery.py --root /home/bfadmin/Documents/datum-eda --enforce --authority-ref <full-owner-promoted-commit> --base-ref <full-G05-complete-commit> --candidate-ref <full-activation-candidate-commit> --environment-path docs/reviews/workflow-delivery-pilot/proof-artifacts/environment.json
```

The current Preferences checkpoint remains GP-CM04, unclaimed, awaiting its own
owner approval for GP-CM05. Completing WDQ must release only this lane; it must
not approve or resume Preferences, enroll its task, change canonical selection,
or schedule a successor. Shared production files remain available only through
the established coordination/ownership rules. This is the proposed resume
disposition, not a request for Preferences approval.

The two explicit nonblocking owner deferrals are now recorded above:
oversized Console records can still hide their tails,
and scrolling beyond available records can show blank history. The reviewer
approved only the finite pilot's complete refusal records. Both beads stay open
for future owning-lane correction; no general Console compliance is claimed.

The exact proposed response below binds the original packet and independently
authored review, whose hash excludes only the future owner-receipt pointer.
The following is the historical request; the uniquely marked receipt and
disposition sections above contain the actual owner response:

```text
ACCEPT WORKFLOW-DELIVERY-GATE-PILOT/WDQ-G06 ab71468a0acd0e845f5e7cd5e2b21aae5121a85dfd5c2d72fa2f5f372c5745a0 b4c214907e43eb4ae7328a005c33e7558f215791928a77cc469edb167b7d1fba
DEFER dat-console-oversized-history-9uz
DEFER dat-console-scroll-range-4y3
```

Alternatively respond `WORKFLOW-DELIVERY-GATE-PILOT: revise — <specific
correction>` or `WORKFLOW-DELIVERY-GATE-PILOT: defer — <reason>`. G06 cannot close
from a bare general instruction to proceed, an agent-authored approval, or the
receipt alone without the owner-controlled promotion and passing enforcement.

## G04 identified producer proof validated

### EVIDENCE:WORKFLOW-DELIVERY-GATE-PILOT:WDQ-G04-NATIVE-PROOF

The corrected native pilot passes producer verification on source input closure
`dde90c754e0a76366a927a0addda60103bd50534`. The clean source archive is retained at
`/home/bfadmin/Documents/datum-wdq-source-oy3bxeo8`.
GUI SHA-256: `2bd1daa4d1a3e8eb393130cd0d543c7f610cfcf3478364827e4e3b807c18f560`.
CLI SHA-256: `80f8a2b1dc741a60b21bab6e397ff82adc57fcef22672e00d662b5608fdbed4b`.
`corrected-build/` records the actual guarded build, complete input manifest and
both executable receipts. All final native runs use these exact binaries and
the archived real-project-derived fixture, on private headless displays/buses.

`proof.json`, `proof-artifacts/`, `producer-assessment.json` and
`corrected-native/PILOT-S01` through `PILOT-S05` bind the five native scenarios,
19 explicit required-dimension assertions, production registry/context exports,
actual dispatches and 392 result-artifact references. The extra
`PILOT-S03-pointer-regression` is bound into S03. All machine checks pass;
producer inspection confirms fitted geometry, retained selection, unavailable
rows, readable complete refusals in opened history, visible menu focus,
context-dependent Fit, normal reopen, and retained native Preferences/Terminal
objects. This is bounded producer proof, not independent review or acceptance.

The standard proof, environment and native-correlation validators passed against
the exact staged Git tree. They checked source-commit/input closure, successful
build receipt, artifact hashes, explicit environment, all five scenarios, all
nine reviewed entry surfaces, and actual production mappings. No trust policy,
enrollment, owner receipt, review verdict or enforcement activation was created.
The owner boundary for GP-CM04/GP-CM05 remains separate.

Remaining findings, deliberately not hidden by this pilot:

- `dat-console-oversized-history-9uz`: one arbitrarily long record taller than
  the panel is still clipped; record-based scrolling cannot reveal its tail.
- `dat-console-scroll-range-4y3`: scrolling past actual history can show an
  empty panel. `corrective-diagnostics/PILOT-S02-wheel-three/` and
  `corrective-diagnostics/before-event-monitor-fix/native/console-narrow/`
  retain that native evidence. The 900-pixel single-pane diagnostic shows full
  refusal readability, but not acceptable overscroll behavior. These findings
  are outside the finite action-readiness assertions and remain open for G05
  independent triage and future owning-lane work, not producer-blessed defects.
- The initial capture monitor used `gdbus monitor` without its required
  destination. Its one-line error was discovered by inspecting the raw log,
  not accepted as an event stream. Those runs are retained under
  `corrective-diagnostics/before-event-monitor-fix/`, outside final proof. The
  corrected owned-bus monitor stays alive across inputs/reopen; all five final
  runs contain real menu StateChanged and ChildrenChanged signals. Tooling
  regressions reject an exited monitor and an error/startup-only log.

Verification: 96 workflow-delivery/tooling tests and 50 selector/claim tests
passed. The earlier three-crate Rust tests and strict Clippy remain on unchanged
production bytes. Source health (1795 files), evidence traceability (20 routes /
114 artifacts), specification governance (203 specs), parity (16 inventories),
menu model, Console boundary, dependency authority, Cargo resource policy and
Frontier/render checks pass. Source/document/JSON whitespace is clean; raw
accessibility logs intentionally retain significant Terminal-cell spaces, which
an unrestricted `git diff --check` reports as trailing whitespace. No log bytes
were trimmed or authority/golden updated to hide a failure. No full drift-suite,
full Console, Preferences, keyboard-only application or EDA-authoring acceptance
is asserted.

## G04 corrective implementation checkpoint

The owner-authorized menu-pointer correction resolves open-menu hits before
canvas focus acquisition, then uses the existing selection/menu dispatcher.
Actual canvas clicks retain their existing focus-and-dispatch path. The history
correction measures wrapped text with the renderer's own fonts and shaping,
allocates per-record height and independent clipping, and selects the newest
complete records that fit. Existing record-based wheel scrolling is preserved.
The compact strip remains intentionally single-line under PM033; opened history
must make the pilot's full refusal explanations readable.

Before implementing Console layout, both complete `gui-feedback-and-status` and
`prototype-command-feedback` routes were reviewed, including Candidate A and B3
DOM/CSS and the owner's PM033 disposition. The protected prototype was rendered
read-only to `/tmp/datum-wdq-correction-5H2qwp/console-reference.png` and inspected.
No authority digest, prototype, golden, setting schema, Terminal path or design
writer changed. Arbitrarily oversized history records remain bounded/clipped;
this is not a general Console history redesign or acceptance of that limitation.

Verification before fresh native capture: protocol 114 unit tests, renderer 156
unit tests, application 305 tests (eight existing ignores), and their enabled
integration/doc tests passed. All-target Clippy for all three crates passed with
warnings denied. Eight focused Console tests include measured wrapping,
independent row clips, record-scroll reachability, narrow widths and scales
1.0/1.25/1.5/2.0. The native runner now names the pointer regression, captures
earlier history after real wheel input, and supports explicit-size diagnostic
captures. These are tooling capabilities, not passing native evidence yet.

## G04 native evidence: blocking defects, not completion

### EVIDENCE:WORKFLOW-DELIVERY-GATE-PILOT:WDQ-G04-NATIVE-BLOCKERS

The production-observation implementation landed in `bd84f3f4`. Native evidence
is under `native/PILOT-S01` through `native/PILOT-S05`, with an additional
`native/PILOT-S03-pointer-regression` reproducer. These are real XTest inputs,
native window captures, live AT-SPI responses/events, actual production dispatch
records, normal-close records and before/after file hashes—not seeded previews.
Per-capture state files retain the latest observed snapshot; complete production
streams remain in the raw GUI logs, avoiding duplicated cumulative traces.
No passing `proof.json`, G04 completion marker, independent replay verdict or
owner acceptance is issued: the required pilot is **not passing**.

Identified build receipts and the complete committed input manifest are in
`native-build-verified/`. Source input closure:
`bd84f3f4e85307d51c7e42ed6768519589a00b1c`.
GUI SHA-256: `608f46e6ed8c37a2f584a148873a13ee119eafff4364069e242ea8d3ead2331b`.
CLI SHA-256: `ecb6760e3d15c999f2410ca3cf4f87aca64316514a5f2bf219c66b275b7b0488`.
The CLI was pinned through `EDA_CLI_BIN`; no implicit Cargo fallback was used.
The source archive is retained at
`/home/bfadmin/Documents/datum-wdq-source-hsci_eyy`. The shared worktree contains
pre-existing ignored runtime/golden artifacts under `crates/`; they were neither
deleted nor committed. Therefore source-clean evidence uses the verified Git
archive, not a misleading claim that `git status` covers every build-root file.

Observed outcomes:

- S01: all three actual Fit entry points reached the same handler, restored the
  scene-bounds fit/zoom 1.0 and preserved the selected pad, layout and other-pane
  camera state. The selected pad was chosen by native pointer input.
- S02: all six missing-action pointer/keyboard attempts refused with zero handler
  invocation. Existing layer toggles and pane navigation remained functional.
  **Visual refusal readability fails**: the transient strip clips the reason,
  and opening Console History reveals overlapping wrapped messages.
- S03: keyboard refusal in the unresolved pane works. The additional native
  pointer test **fails**: clicking visibly disabled Fit changes focus from pane 1
  to pane 0 and invokes Fit against Board. Board zoom changes from
  `1.5735193490982056` to `1.0`, and the camera is recentered despite the displayed
  unavailable context. This is not an acceptable availability/dispatch match.
- S04: normal close/reopen and source/journal preservation were observed. New
  paths are enumerated Terminal context/session/runtime sidecars, not new design
  partitions or journal transactions. No Save/undo/crash-recovery claim is made.
- S05: live native identities, focus, availability, descriptions and dismissal
  were captured, including the existing Terminal and the separate native Global
  Preferences window. This does not accept the Preferences implementation or
  excuse the S02/S03 failures.

`native-observations.json` records machine-check findings, explicitly not visual
quality or acceptance. The pointer regression must remain red until corrected.
Producer visual inspection of `native/PILOT-S02/refusal-history.png` is the
readability failure evidence; a machine-check pass cannot override it.

### Bounded corrective handoff authorized after blocker capture

On 2026-09-06 the owner answered `please continue` to the explicit request to
extend this lane to `runtime_primary_pointer.rs` and `datum_console.rs` and rerun
native evidence. This authorizes only the two corrections and focused proof
below. The synchronized WDQ lease records that scope; GP-CM04/GP-CM05 and G06
remain separate owner boundaries. The following request describes the boundary
presented before this approval, not a continuing hold.

Two newly implicated shared production files are outside the current declared
WDQ file scope. Before edits, obtain the owner/consumer handoff, review each full
owning evidence route and synchronize the Frontier/beads lease. No change to the
pilot's expected behavior, source-health policy, authority digests or prototypes
is requested.

| Finding | Exact correction boundary | Preserved behavior and required proof |
| --- | --- | --- |
| `dat-menu-pointer-pane-authority-sjn` | `crates/gui-app/src/runtime_primary_pointer.rs`, `handle_primary_click` pane-focus work before overlay dispatch; focused regression tests | A menu hit must not acquire the underlying pane before action admission. Preserve genuine canvas click-to-focus, Terminal/Preferences input ownership and the single camera resolver. Repeat disabled pointer Fit with pane 1 retained, no invocation and unchanged Board camera; rerun all three supported Fit paths. |
| `dat-console-history-wrap-overlap-lkp` | `crates/gui-render/src/datum_console.rs`, history row measurement/advance and clipping; focused rendering tests | Make full refusal explanations readable at the reviewed split-pane size without overlapping adjacent rows. Preserve Console/Terminal separation, existing settings, manual history controls and Claude-owned visual authority. Prove native readability plus narrow-layout, scrolling and existing-consumer regressions. |

The Preferences session need not resume general development to investigate these
two fixes. Its GP-CM04/GP-CM05 owner boundary remains separate. Either grant this
lane the exact corrective scope or hand the findings to an owning lane and return
verified commits. G04 stays in progress; G05/G06 remain pending and enforcement
stays off. Shared accessibility work is already committed in `4174fa24`; the
observation hooks are committed in `bd84f3f4`. No uncommitted shared Rust changes
are part of this evidence handoff.

### Capture-tool corrections and retained diagnostics

The source-archive wrapper initially supplied Cargo's target override after the
guard boundary, so the guard measured another target. The caller now gives
`--target-dir` to the guard itself and the final verified receipt comes from the
correctly configured rerun. `dat-cargo-target-preflight-olt` captures the broader
guard hardening follow-on; no resource policy was relaxed.

The harness initially inherited an accessibility activation runtime directory,
allowing concurrent private-session runs to collide on an AT-SPI socket. It now
creates its own session bus, updates only that bus's activation environment, and
rejects accessibility addresses outside its private runtime directory. The final
records identify run-specific private sockets. Earlier collided runs are not
used as final evidence. Capture also checks stable live semantics across the
image and records all native windows: Preferences is a separate window, so an
unchanged main-window capture alone is insufficient evidence of its appearance.
Native dialog focus is explicit before sending its Escape key.

Earlier diagnostic runs/receipts were moved intact to
`/tmp/datum-wdq-g04-observation-oznPmN/`; nothing there was deleted or promoted to
passing proof. The capture/build/observation tooling does not issue acceptance.

Checkpoint verification: 90 workflow delivery/tooling tests and 50 selector/claim
tests passed. Source health (1792 files), evidence traceability (20 routes/114
artifacts), specification governance (203 specs), dependency authority, Cargo
resource policy, Frontier validation/render and whitespace checks passed. The
native observation evaluator deliberately exits 1 for the reproduced pointer
scope failure; Console readability remains a separate producer visual failure.
Native Preferences dismissal was targeted to its real window; subsequent View
menu use and retained Terminal accessibility were observed without settings-file
changes. These results do not claim a passing full drift suite or G04 acceptance.

## G04 shared accessibility handoff authorized

Recorded 2026-09-06 19:42 UTC at clean HEAD
`3f377a7a61a3691e9ede564d8fe4ea3b83ce8ba5`. Preferences completed GP-CM03 and
released its claim; GP-CM04 remains an owner decision before GP-CM05. The owner
reported the other session paused, then answered `the working tree is clean.
please proceed.` to the explicit request for temporary WDQ-G04 ownership of the
held shared accessibility files, preservation of Terminal/Preferences behavior,
and the other session leaving those files untouched until handback.

This releases only the previously held native accessibility group for additive
menu publication, plus its focused modules/tests and frame-refresh connection.
It supersedes the historical hold statements below. The live WDQ lease records
the exact scope. No GP-CM05 approval, Preferences task transfer, prototype edit,
dependency addition or G06 activation is implied. At handback, record commits,
validation, remaining defects and whether any shared files are still dirty;
Preferences may resume only under its own owner authorization and file handoff.

### G04 production observation and capture tooling checkpoint

The next bounded implementation adds `DATUM_ACTION_EVIDENCE=1` observations to
the real menu pointer, menu keyboard and editor-shortcut call sites. The stream
exports the actual production registry for observed contexts, records invocation
only at dispatch, and records focus, selection, layout, pane cameras, scene
bounds, layer filters and Console feedback. It writes diagnostic stderr only;
there is no injected action, new project writer, readiness override or acceptance
flag. Disabled diagnostics do not build snapshots. A broken diagnostic pipe
cannot panic the application.

`scripts/workflow_delivery_pilot_capture.py` runs explicit physical inputs on a
private Weston/Xwayland display and accessibility bus, using installed system
tools. It pins the CLI executable, copies the archived native fixture, records
binary hashes and raw observations, sends normal WM_DELETE_WINDOW close requests,
and enumerates changed/removed and newly created project paths. It neither
installs dependencies nor produces an acceptance verdict. The sibling scenario
module records the inspected scale-one input coordinates; the X11 helper follows
the installed Xlib ABI. No prototype changes or Preferences approval are included.

Proof before collecting final scenario evidence: GUI app tests 305 passed with
eight existing tests ignored; strict all-target app Clippy passed; workflow
delivery/tooling tests 86 passed. The isolated harness smoke opened View,
collected actual native menu and Terminal accessibility nodes, captured the
native window and closed with exit zero. Archived files were unchanged; runtime
sidecars were enumerated. The smoke was on development bytes and is not a final
PILOT-S01–S05 result. G04 remains in progress pending identified scenario evidence.

### G04 additive menu accessibility implementation checkpoint

The bounded shared-file change projects currently open production menu rows
through the existing Linux accessibility service. Stable menu identities,
labels, focus, enabled/sensitive state and unavailable descriptions derive from
the actual menu inventory and contextual action admission. Dismissed menu paths
fail closed. Menu publication retains independently cached Terminal snapshots,
Preferences/New Project nodes and Console announcements; no settings or design
writer was added. Menu accessibility is read-only, not a new Action invocation
surface or a whole-application keyboard-only access claim.

Read-only implementation review identified one blocking event-index defect:
coalescing menu dismissal with Preferences publication used the new root sibling
offset for the removed menu. The worker now retains previous and next offsets;
removals use the previous index and additions use the next. A focused regression
covers both directions. This review is not WDQ-G05 independent native replay.

Verification at this checkpoint:

- Guarded offline/locked GUI app, protocol and render library/binary tests:
  571 passed (303 app, 114 protocol, 154 render), eight existing app tests ignored.
- Strengthened the publication regression to retain nonempty Preferences nodes,
  then reran all accessibility tests inside a private `dbus-run-session` with
  desktop display variables unset and `--include-ignored`: 30 passed, zero
  ignored, including real accessibility-bus registration. No desktop GUI opened.
- Guarded offline/locked Clippy for the three GUI packages, all targets with
  `-D warnings`: passed.
- Workflow delivery validator tests: 82 passed. Selector/claim tests: 50 passed.
  Source-health regression tests: 13 passed.
- Source health: 1785 files passed; evidence traceability: 20 routes/114 artifacts
  passed; dependency authority, specification governance (203 specs), project
  state (50 Frontier items), generated Frontier and whitespace checks passed.

The initial Cargo resource preflight refused insufficient `/tmp` reserve. With
explicit owner authorization, inactive `/tmp/datum-ai-disc-target` compiler
artifacts were moved intact to
`/home/bfadmin/Documents/datum-build-archive-NZ8vkV/datum-ai-disc-target`, freeing
4.2 GiB; nothing was deleted and no resource policy was bypassed.

This is an implementation checkpoint only. Native PILOT-S01–S05, production
registry/dispatch evidence export, identified binary/input receipts and complete
`proof.json` remain outstanding. G04 stays in progress, G05/G06 are not advanced,
and enforcement stays off. The bounded shared accessibility lease remains with
WDQ pending native verification and explicit handback; committing this checkpoint
does not approve GP-CM05 or release overlapping files to concurrent edits.

This is an operational record under the ratified PM041 gate contract and
WORKFLOW_DELIVERY_ADOPTION_PACKET, not a new product specification. It is outside
the pilot authority route: future session/receipt updates must not create a
proof/authority hash cycle. Candidate behavior is in
`specs/workflow_delivery/pilot.contract.json`, committed in `b64a79ba`.

## Current ownership observed

At `5c6ce3d9b0247ea1a36ad0cf7fe70a80c131fb81`, the worktree was clean. The
successful named selector reported WDQ-G01 in progress, planning authorization,
with live session `codex-wdq-g01-planning-20260906`. A clean worktree is not a
release of another session's claim.

GLOBAL-PREFERENCES-COMPLETION retained its canonical execution claim under
`codex-gp-cm03-execution-20260905`. Its declared scope is the GP-CM03 product
service, trusted-human/proposal authority, CLI/MCP/daemon parity, writer model
and eight-key Project Units genesis. Nothing here pauses, completes or transfers
that work. Refresh both claims and dirty/staged paths at the actual G02 window.

## Proposed implementation file handoff, not a lease

| Responsibility | Paths and boundary |
| --- | --- |
| Delivery validator | New scripts/workflow_delivery_contract.py, scripts/check_workflow_delivery.py and focused scripts/test_workflow_delivery_*.py, as already specified by the adoption packet; no code written in G01 |
| Selector integration | scripts/project_status.py and scripts/project_task_details.py; preserve selection, claims and byte-for-byte output; focused new tests instead of growing test_project_status.py |
| Staged/standing checks | scripts/git-hooks/pre-commit, scripts/run_drift_gates.sh, .github/workflows/alignment.yml; preserve every existing check and use the trusted revision for final enforcement |
| Actual action admission | crates/gui-protocol/src/gui_menu_model.rs; crates/gui-app/src/runtime_menu_actions.rs, runtime_view_actions.rs, workspace_keyboard.rs; crates/gui-render/src/menu_chrome.rs. One production readiness source must serve dispatch, paint and accessibility |
| Camera eligibility integration | crates/gui-app/src/runtime_camera_pane.rs; existing focused-camera resolver, not a separate menu-only context heuristic. Source-health review/extraction is required before touching oversized code |
| Shared native accessibility handoff | crates/gui-app/src/terminal_accessibility_bridge.rs and terminal_accessibility_platform/{worker.rs,atspi.rs,events.rs,atspi/properties.rs,atspi/introspection.rs,atspi_tests.rs}; exact final diff scope requires the current consumer owner's agreement and complete owning-route review before edits |
| Visual truth | No prototype change proposed. All docs/gui/prototypes/*.html remain exclusively in Claude's lane; no marker transfer or golden/digest bypass |

The current planning lease authorizes none of those implementation edits.
Module additions, registration changes and source-health extractions must be
named in the final implementation lease, not assumed from this starting map.
The inactive FitBoard arm in main.rs does not justify editing that monolith or
restoring its removed button. Preferences runtime, engine, CLI and MCP semantics
remain with their owners; no blanket unknown-action disablement is permitted.

## Hook authority review completed

Read the complete visual-truth-file-lane-governance route: its enforcement
packet, AGENTS.md, pre-commit hook, file-lane checker and tests. Reviewed route
digest: 83178dae95c68fbe991b71b56ade930fb0265f3173ebb19681f318acc1d53b5c.
No route member was edited and no authority digest was refreshed.

The configured local hooksPath is scripts/git-hooks. The hook runs staged lane
validation before exec'ing staged rustfmt. A future third check must preserve
both checks and their failure behavior; it cannot be appended after an exec and
claimed reachable. It must inspect index bytes, not another session's unstaged
work. Existing test coverage includes nonmutation and lane-before-format order;
this planning review did not execute or alter the protected-lane marker tests.

## Trusted-runner configuration handoff

Inspection found no WDQ invocation or authority/base input in the checked-in
hook, drift runner or alignment workflow. No externally configured owner trust
was verified. The existing source-health CI base is not WDQ approval authority.

Use the PM041 fallback: bootstrap remains report-only under existing blocking
gates, with independent owner-run verification until an owner-controlled runner
is established. No CI enforcement or tamper-proof approval claim is justified.
At G06 the owner selects/promotes the exact trusted authority commit externally;
the trusted runner executes that revision's validator against the candidate and
receives the comparison base independently. Never derive authority from candidate
HEAD, repository policy fields or an implementing session's chosen receipt.
No trust values, Git settings, CI variables or activation flags are installed here.

## Session separation at the initial handoff

- Planning session: codex-wdq-g01-planning-20260906.
- Proposed implementation/original-proof producer: this session only if explicitly
  named in the final G02 handoff and synchronized execution lease. No execution
  has occurred and a future session change must be recorded before implementation.
- Independent replay session/person: **not assigned**. The owner was asked to
  identify it. It must not participate in implementation or original proof, must
  replay every mandatory native scenario, and must inspect input closure and N/A
  dispositions. A fabricated reviewer ID is not a handoff.
- Current Preferences owner handoff: **not confirmed**. At G02 record its bounded
  commit, exact continue-on-nonoverlapping-files or pause disposition, retained
  step/claim and resume condition. Historical GP-CM01 is not that checkpoint.
- Trusted owner-selected authority/base values: intentionally not set during
  bootstrap; only the owner-controlled boundary can promote them at activation.

The unassigned/unconfirmed entries above describe the initial ed1ef716 handoff.
The owner disposition below supersedes those two missing coordination choices.

<!-- EVIDENCE:WORKFLOW-DELIVERY-GATE-PILOT:WDQ-G01-HANDOFF -->
## Owner-reviewed coordination and completed planning handoff

The owner first replied `I reviewed the handoff markdown please proceed.`
This session then asked: `One coordination choice remains before I can close
WDQ-G01: may I use a fresh, independent review-only session and let the Preferences
session continue on nonoverlapping files, with shared accessibility files held
until an explicit handoff?` The owner replied exactly `yeah, peoceed`.
Recorded 2026-09-06 UTC from this conversation. This approves the reviewed
handoff and that coordination arrangement, not unperformed proof or G02 execution.

- Independent lane actually created: `/root/wdq_independent_review`, fresh context,
  review only. Receipt-facing session label: `wdq-independent-review-20260906`;
  this label maps to that actual lane, not a fictional reviewer. It acknowledged
  document intake only, no implementation/original proof, no file/tracker edits
  and no owner approval. No blocker to reservation was reported. G05 is not started.
- Intended implementation/original-proof producer remains
  `codex-wdq-g01-planning-20260906` (this session), subject to G02 execution
  authorization and its synchronized lease. Any replacement of either session
  requires a recorded handoff before participation; separation must be preserved.
- Preferences continues under its unchanged claim on nonoverlapping files.
  At this transaction its daemon dispatch.rs, main.rs and preferences_state.rs
  changes were already staged by the other lane; none belongs to this commit.
  No blanket pause, completion claim or canonical-task transfer occurs.
- The complete shared accessibility path group in the table remains held,
  excluded from an executable lease until an explicit bounded handoff with its
  consumer owner and complete owning-route review. A future newly overlapping
  path also stops at that boundary. This is the approved reservation, not a claim
  that the other session has already transferred those files.
- Initial implementation window proposed for G02: report-only G03 validator and
  selector integration on the listed nonoverlapping script/governance paths.
  G04 native work must observe the held-file condition; no native acceptance can
  skip S05 because accessibility work is held. Preferences needs no resume action
  while it continues; if an actual overlap later requires a pause, record the
  bounded commit, retained step/claim and explicit resume condition first.
- Trusted-runner disposition is the report-only/owner-run fallback above. No
  candidate-selected trust values or CI enforcement are installed. G06 must
  establish the external owner-controlled trust boundary before activation.

G01 completes its planning deliverables: ratified contract and real input paths,
four-consumer/five-scenario candidate, reviewed hook authority, named independent
lane, file reservations and trusted-runner fallback. It does not assert native
readiness, completed implementation or acceptance. The sole selected successor
is G02 for the exact bounded implementation authorization, including review of
the candidate's keyboard-access scope limit. G06 remains the separate activation
decision. The parent issue remains open.

<!-- EVIDENCE:WORKFLOW-DELIVERY-GATE-PILOT:WDQ-G02-AUTHORIZED -->
## Bounded implementation authorization

The owner replied exactly `WORKFLOW-DELIVERY-GATE-PILOT: authorize bounded implementation`
in this conversation on 2026-09-06 UTC, after G01 closure commit 9e04a702 and
presentation of the G02 decision packet. This completes G02 and authorizes the
reviewed G03–G05 build/proof scope with G03 as the sole selected execution step.
G06 remains the separate blocking activation/acceptance decision.

The current implementation window begins from 46764e7cb042cd65e4c0eafcfb6aec9e3901015d,
with a clean worktree observed before claiming. Preferences retains its own
canonical claim and continues on nonoverlapping files; there is no pause to resume.
Shared accessibility files remain held exactly as approved above. Any overlapping
change requires an explicit bounded handoff before editing; authorization does
not erase that condition or allow S05 to be waived.

Implementation/original-proof session: codex-wdq-g01-planning-20260906.
Independent replay lane: /root/wdq_independent_review, review only, as mapped above.
The execution lease starts with focused dependency-free JSON/path/hash input
handling in scripts/workflow_delivery_io.py and scripts/test_workflow_delivery_io.py,
then the already reviewed schema/evidence/trust modules and selector integration.
Neither schema migration nor validator completion is asserted by this authorization.

### G03 first implementation unit (not completion evidence)

`scripts/workflow_delivery_io.py` now provides read-only strict UTF-8 JSON parsing,
canonical JSON and raw-byte hashing, normalized repository paths, and contained
regular-file reads. Duplicate keys, nonfinite numbers, lone surrogates, path
escapes, missing files and broken/cyclic symlinks are refused. The companion
test module passes 13 hermetic tests; these are input primitive tests, not a
claim that N01–N20/P01–P07 have passed.

Verification for this unit: 45 project-status regression tests and 13 source-health
governance regression tests pass; project-status (50 items), evidence traceability
(20 routes / 114 artifacts), spec governance and progress coverage checks pass.
Source-health checks pass. No Rust build or native GUI proof was run for this
dependency-free Python unit.

G03 remains in progress. Closed-shape contract validation, evidence and freshness
validation, independent-review/owner-receipt validation, trusted revision and Git
index handling, report-only CLI, selector integration and the full refusal/positive
matrix remain outstanding. No hook, policy enrollment, Frontier schema, GUI
behavior or acceptance authority changed in this unit.

<!-- EVIDENCE:WORKFLOW-DELIVERY-GATE-PILOT:WDQ-G03-VALIDATED -->
## Report-only delivery implementation and hermetic validation

Recorded 2026-09-06 UTC. This supersedes the first-unit outstanding-work list
above. The validator and integration are implemented; no production registry,
native PILOT-S01–S05 run, G05 replay or G06 activation is asserted here.

The dependency-free implementation separates closed contract/evidence shapes,
authority resolution, Git candidate views, input/build freshness, typed evidence
correlation, independent replay/owner receipts and Frontier integration. New
modules remain below the 350-line target; existing selector modules remain below
700 lines, with no legacy source debt or dependency addition.

Working-tree, staged-index and explicit-commit views are separate. Required
artifacts must be present in the candidate Git view; untracked production inputs
are included in worktree input manifests. Index reads never consult unstaged
files. Full PM025 validation/projection is reused in an isolated temporary
metadata view for enforcement; candidate files, index, configuration and authority
records are not written. Temporary metadata is removed on exit.

### Implemented evidence transport

These are the validator/runner serialization interfaces implementing PM041, not
additional product decisions or evidence that the native producer exists:

- A hashed result artifact with `kind: datum.workflow-delivery.artifacts/v1`
  binds every sibling artifact to explicit events/captures/state/registry roles.
  Event roles—not filenames or identical screenshots—control replay independence.
- Event records bind scenario, producer, input method/actions, binary and current
  authority hashes to production dispatch observations and visible/state results.
  Registry records bind tested binary, actual keys/handlers, entry surfaces and
  contextual eligibility. G04 must emit these from production, not maintain an
  independent contract-shaped registry or substitute CLI edits for native input.
- The requested environment is supplied explicitly by `--environment-path` and
  compared exactly with the proof environment: OS, backend, toolchain, scale,
  input method, window size and reproduction commands. No equivalence is inferred.
- Owner receipts are extracted only from the uniquely marked governed section,
  containing the exact ACCEPT line plus `Source:` and `Date:`. Trusted defect
  dispositions explicitly name `RESOLVED <issue>` or `DEFER <issue>`; a blocking
  resolution also binds `REPLAY <replay-blob-sha256>` and requires passing replay.
- Enforcement requires full owner-selected authority/base commit IDs and the
  trusted revision's gate bytes. Candidate policy/gate/contract weakening,
  disappearing enrollment and regressed completed obligations refuse. Changes
  to reviewed build inputs cannot evade proof by leaving activation pending.

Local hook order remains lane gate, staged rustfmt, then staged **report-only**
delivery diagnostics. Existing blocking behavior is preserved. Drift runs add the
hermetic suite and report-only diagnostics; CI adds no owner trust value or
enforcement switch. G06 must establish the external trusted runner/promotion.
The optional owner-controlled clone configuration keys consumed by ordinary
selector checks are `datum.workflowDeliveryAuthorityRef`,
`datum.workflowDeliveryBaseRef` and `datum.workflowDeliveryEnvironmentPath`;
none has been installed by this lane.

### Verification and limits

`python3 -m unittest discover -s scripts -p 'test_workflow_delivery_*.py'`
passes 82 tests. The N/P matrix is exercised with tiny hermetic records; native
production-path demonstration remains G04, and independent native replay remains
G05. Tests do not certify that prose N/A dispositions or screenshots are truthful.

| Refusal/positive cases | Focused test ownership |
| --- | --- |
| N01–N04; P01 | Input, contract, authority and CLI tests |
| N05–N08; P02–P04 | Typed production-consumer/correlation protocol tests; synthetic, not native runs |
| N09–N12, N19; P05 | Artifact/input/build freshness, index/worktree and authority-change tests |
| N13–N15; P06 | Distinct replay, same fixture/binary, copied-event refusal, exact trusted receipt and defect disposition tests |
| N16–N18; P07 | Full Frontier/claim/selection, retained enrollment/completion, candidate tampering and explicit trust tests |
| N20 | Explicit requested-environment mismatch tests; no implicit equivalence |

Additionally: 45 existing project-status tests, five claim-state tests and 13
source-health governance tests pass. Evidence traceability (20 routes / 114
artifacts), spec governance, progress coverage, project status (50 items), Cargo
resource policy and whitespace checks pass. Source-health passed at 1,763 files
after the other lane corrected its transient oversized Preferences module.
Schema-5/6 legacy next/details output is tested byte-for-byte unchanged.

The full drift battery was attempted: delivery tests and Cargo-resource checks
passed, then it stopped at Preferences-owned rustfmt failures in
`crates/engine/src/preferences/mod.rs` and `project_genesis.rs`. A separate parity
check still reports the concurrent engine API inventory at 203 versus its
documented 201. This is **not** a green full-repository or native-GUI verdict.
Those files and their parity authority remain with the Preferences owner; this
lane has not formatted them, changed their inventory or absorbed their failure.

The reserved independent review lane performed read-only code inspection. Its
findings about completion regression, disappearing enrollment, fixture/binary
identity, defect disposition and full Frontier validation were corrected with
regressions. Its bounded final recheck found no remaining blocker in those fixes;
it did not run tests, produce original proof or perform G05 replay/acceptance.

### G03 landing and synchronized continuation

Implementation landed in `9cb42473`. The completion transaction records its
committed validation evidence, migrates the live Frontier to supported schema 6
without adding any delivery enrollment, releases the finished G03 lease and
selects G04 pending. Existing task history, canonical Preferences selection and
other claims are unchanged. The pilot issue remains open under the already
recorded G02 build/proof authorization; no new approval is inferred.

Before claiming G04, refresh the production file map and declare its exact scope.
Editing any held shared accessibility file still requires the explicit bounded
consumer-owner handoff; a nonoverlapping scope does not release that hold.
The separate reviewer remains reserved for G05 native replay. G06 still requires
owner acceptance and external trusted-runner promotion; report-only operation
must not be described as activated enforcement.

## G04 production admission: first implementation unit

Recorded 2026-09-06 UTC. The owner replied `pkease proceed` after confirming
that the other session was still developing Preferences. The refreshed selector
selected G04 with execution authorization and no claim. The synchronized lease
now names the menu/protocol/runtime/render files and the shared viewport camera
resolver; the implementation/original-proof session remains unchanged. No
Preferences claim, selected step or authorization was transferred.

The production registry owns exactly the four reviewed pilot keys, typed Fit
handler identity, actual entry-surface identities and stable unavailable reasons.
Menu paint and invocation consult the same admission function; editor F reaches
the same dispatch instead of directly bypassing it. Missing consumers stay
unavailable, and missing registry keys never acquire handlers. Other GUI-local
families retain their existing dispatch ownership, including Preferences, pane
navigation and individual layer controls.

Shared viewport camera-scene resolution lives in `workspace_interaction/camera.rs`,
not in the menu model. Readiness and actual camera routing consume that resolver;
hidden, stale or unresolved panes cannot silently target another Board. Menu
activation/refusal restores editor focus before a supported action establishes
any new overlay focus. Existing Preferences submenu navigation tests remain in
place. No prototype, golden, menu inventory, dependency or source-health policy
changed. All touched source modules remain within normal budgets.

The reserved independent reviewer inspected this bounded diff read-only and
reported no concrete blocking defect. It ran no tests or native input and did
not perform G05 replay or acceptance.

### Remaining work and held ownership

This unit is not G04 completion. Production registry export, native event/state
correlation, accessible menu publication, proof-grade GUI/CLI build receipts and all
PILOT-S01–S05 native evidence remain outstanding. Unit/render construction tests
do not replace those observations. No `proof.json`, `review.json`, policy
enrollment, trusted setting or owner acceptance is created here.

The owner was asked to obtain a bounded Preferences consumer-owner handoff for
the shared accessibility files already listed above, naming its current commit,
retained ownership and in-flight changes. Those files remain untouched and held;
this request is not a release. The other session may continue on nonoverlapping
files. No native window has been opened on the owner's desktop.

The pre-existing menu CSV round-trip failure was reproduced without writes:
`menu_model_csv.build_obj()` omits the live `edit.preferences` submenu, and its
column/entry schema cannot retain `requires_project`. Captured as
`dat-menu-csv-roundtrip-etp`; blindly following the checker's regeneration advice
would remove current Preferences semantics. The owning menu/Preferences lane
must reconcile that separately. Neither CSV/JSON nor generator was edited here.

### First-unit verification

Guarded, offline/locked tests for the application, protocol and renderer passed
558 library/binary tests: 294 application, 112 protocol and 152 renderer;
eight existing application tests were ignored, not passed. The final rerun used
`cargo test --offline --locked -q -p datum-gui-protocol -p datum-gui-render
-p datum-gui-app --lib --bins` through the guarded proof runner. The
focused runs also passed 11 menu protocol tests and 18 application/renderer menu
tests. Coverage includes real menu inventory mapping, exact pilot handler/reason
identities, unresolved/hidden/stale camera contexts, inspectable unavailable rows,
rendered enabled/disabled state and existing Preferences submenu navigation.
These are unit/construction tests, not native user-input proof.

Delivery tests passed 82; existing project-status tests passed 45, claim tests
five and source-health regressions 13. Source health, evidence traceability
(20 routes / 114 artifacts), spec governance (203 classified), progress coverage,
project-state/projection, menu-model validity and dependency/Cargo-resource policy
checks passed. No authority digest was refreshed.

Full drift was attempted: delivery and Cargo-resource checks plus workspace
rustfmt passed, then workspace Clippy stopped with 33 Preferences-owned engine
diagnostics (large error/enum payloads and repository open options). This is
ongoing other-lane work, not a completed Preferences verdict or a WDQ repair
authorization. The separate CSV round-trip failure above is also unresolved.
No full-repository green claim is made.

### Offscreen render inspection, not native proof

The guarded command `cargo build --offline --locked -p datum-gui-app
-p datum-eda-cli --features datum-gui-app/visual` passed. Diagnostic binary hashes:

- GUI: `ecec45fea1746c89b48c0849eb5d1154d9f5f91c52b609c0f94aafdb5223484d`.
- CLI: `32b79c2f0addf2cefd6d8eb1f979c50bfb2b469d8e6bd0df21568f3a014900be`.

The existing C01 archive was extracted under the isolated temporary directory
`/tmp/datum-wdq-g04-preview-ptukH1/project`. Captures used that Project, pinned
`EDA_CLI_BIN` to the just-built CLI, private XDG config/runtime directories,
unset DISPLAY/WAYLAND_DISPLAY, and `--visual-test --exit-after-screenshot
--window-size 1280x768`. Help used `--open-menu Help`; the unresolved schematic
capture used `--open-menu View --focus-pane schematic`. No native input was sent.

Both [Help preview](g04-help-preview.png) and
[unresolved schematic View preview](g04-schematic-preview.png) were inspected.
About and the ineligible Fit row are muted, focus outlines remain visible and
the existing menu/pane geometry is preserved. The displayed fixture revision
is not a build identity. `tar --compare` confirmed all archived source/journal
bytes unchanged after both captures; this is not a complete new-path or native
reopen audit. Concurrent Preferences inputs were not frozen into a full proof
manifest; these previews must never substitute for fresh PILOT-S01–S05 evidence.
