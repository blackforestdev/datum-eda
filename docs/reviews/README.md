# Review records and temporary proof output

## Documentation fragmentation audit and first consolidation

Owner-directed 2026-10-14 audit and justified consolidation/removal. This
landing covers structural inventory and the first verified retirements; it
is not a claim that all semantic documentation duplication has been resolved.
S5A remains paused, with product requirements and approval records unchanged.

<!-- REQ:DOCUMENTATION-CONSOLIDATION:DOC-A01 -->
- DOC-A01: Inventory all tracked text documents and identify reference/duplicate candidates.
<!-- REQ:DOCUMENTATION-CONSOLIDATION:DOC-A02 -->
- DOC-A02: Integrate the redundant M4 shard and retire three obsolete M0/M1 execution documents preserving historical retrieval.
<!-- REQ:DOCUMENTATION-CONSOLIDATION:DOC-A03 -->
- DOC-A03: Explain documentation directory roles and target contents; record unresolved semantic candidates separately.

<!-- EVIDENCE:DOCUMENTATION-CONSOLIDATION:DOC-A01 -->
At source 8d0e8ca4, inventoried all 515 tracked .md/.rst/.txt documents and
scanned references in all 5,921 readable tracked UTF-8 files (43 binary or
unreadable files excluded from text scanning). The disposable per-file CSV and
JSON audit are /tmp/datum-documentation-audit-20261014.{csv,json}; no raw audit
bundle is added to Git. Found 33 pairs sharing at least three substantive
paragraphs and 110 documents without detected narrative inbound references.
These are structural review candidates, not proof of semantic redundancy.
There are no byte-identical Markdown documents; eight identical text-capture
groups are historical evidence. Required license copies are retained.
Of the 515 documents, 215 have no direct governance entry; this includes
research, reviews, licenses and companion notes outside mandatory globs and
must not be interpreted as 215 governance failures or deletion approvals.

<!-- EVIDENCE:DOCUMENTATION-CONSOLIDATION:DOC-A02 -->
Removed docs/workflows/m4_live_slice.md: every capability/limitation already
exists in specs/progress/m4_details.md, including the direct pool-ref query,
replacement artifacts and schema-only component_silkscreen_texts. The scenario
document now links to that sole detail owner and its duplicated live-status
block is removed. Original launch exclusions and placeholder commands are
explicitly historical; unique scenario rationale is retained/classified.
The empty docs/workflows directory is removed.

Removed three explicitly historical execution documents from the current tree:
docs/M0_TASK_CHECKLIST.md, docs/M1_TASK_CHECKLIST.md and
docs/IMPLEMENTATION_GUARDRAILS.md. Their completed/obsolete milestone sequencing
and placeholder restrictions are not current requirements. Each complete source
remains retrievable with `git show 8d0e8ca4:<original-path>`. The retired M0
checklist also contained a historical closure assessment: it reported pool
types/serialization, rule validation, SQLite indexing, import identity and
Eagle library import implemented; original M0 exit criteria satisfied with 22
.lbr fixtures; further golden expansion belonged to M1 rather than blocking
M0 closure. Those dated assertions are preserved here and in the pinned source;
they are not rerun, newly accepted or promoted to current qualification.
No unique live product rule, current acceptance receipt or owning
evidence-route artifact is removed.
References among these three disappear together; the old repository-tree list
in the explicitly historical architecture snapshot is preserved as history,
not refreshed as current layout or used to authorize obsolete work.

<!-- EVIDENCE:DOCUMENTATION-CONSOLIDATION:DOC-A03 -->
The directory roles below distinguish current authority from rationale,
execution history and generated output. Directory placement alone does not
make documents redundant or confer product authority.

| Location | Purpose and disposition |
| --- | --- |
| docs/contracts/ | Seven domain/shared interaction contracts: retain current behavior requirements. |
| docs/decisions/ | Sixty-nine numbered decisions/amendments: preserve ratified choices and explicit supersession history. |
| specs/ | Detailed schemas/contracts and the canonical structured Frontier/current status; not a second source of product doctrine. |
| specs/progress/ | Detailed implementation notes linked from PROGRESS, rather than duplicated capability ledgers in scenario docs. |
| docs/gui/ | GUI requirements/guidance and protected visual sources; retain owning authority and review before consolidation. |
| research/ | Decision inputs with checked owning routes; unique rationale is not automatically redundant after spec integration. |
| docs/audits/ and docs/reviews/ | Dated findings, owner/reviewer receipts and evidence. Preserve acceptance/provenance records; retire disposable capture detail only after reference review. |
| docs/ root | Product/operational entry points plus legacy architecture/planning material; duplicate status and obsolete execution controls are consolidation candidates. |
| target/ | Ignored local Rust output, currently also holding historical native fixtures. Not tracked source or part of pushes. |

At the audit boundary docs/reviews had 2,589 tracked files, 57.7 MiB of file
content (155 text documents plus structured reports/records). File count,
required decision records, rationale and current requirements are separate
questions; merging all of them would erase useful roles without resolving
contradictions. Identical license texts remain distributed where required.

Target occupies 2.78 GiB allocated: debug holds 1.33 GiB in the retained CLI,
GUI and engine-daemon binaries (unstripped ELF files with debug information),
with approximately 1.45 GiB in fixture/proof
collections. It has zero tracked files. Its 568 native project roots include
referenced/current S5A and user-edit inputs, not just rebuildable cache. The
793 MiB S5A inventory alone contains 81 projects. Do not use `cargo clean` or
blanket target removal while these inputs are mixed with build output. The
existing guarded build-only pruner retains them; newly managed S5A fixtures
already default outside the checkout. Migration of referenced inputs must
preserve hashes, location/recovery records and replay references first.
No target data was deleted or moved in this documentation pass.

Remaining semantic reconciliation is explicitly tracked as dat-documentation-semantic-reconciliation-olnn.
It covers repeated IR/selection and performance clauses, rationale/spec pairs,
retired execution plans and June audits, stale dependency/launch assumptions,
review capture duplication and nonportable Markdown links. Complete owning
routes and exact consumers must be reviewed before each such edit. Current
contracts/decisions/owner receipts remain; no automatic intake successor or
S5A resumption is implied. Structural inventory does not establish that every
fragment has been consumed or safely removed.

## Bounded specification and workspace cleanup

The owner selected these five improvements on 2026-10-14. Current product
requirements and recorded approvals remain controlling; S5A stays paused.
Changes require independent review and truthful fixture/qualification boundaries.

<!-- REQ:SPEC-WORKSPACE-CLEANUP:SPEC-01 -->
1. Clarify the existing specification entry point and subject ownership/status.
<!-- REQ:SPEC-WORKSPACE-CLEANUP:SPEC-02 -->
2. Separate current completion requirements from chronological execution records, preserving references and approvals.
<!-- REQ:SPEC-WORKSPACE-CLEANUP:SPEC-03 -->
3. Reconcile inherited precedence descriptions and repeated rules without changing product scope.
<!-- REQ:SPEC-WORKSPACE-CLEANUP:SPEC-04 -->
4. Audit preserved fixture directories and execute justified retirement dispositions preserving native user data and current evidence.
<!-- REQ:SPEC-WORKSPACE-CLEANUP:SPEC-05 -->
5. Standardize temporary project generators and user examples outside the source checkout with ownership and cleanup rules.

<!-- EVIDENCE:SPEC-WORKSPACE-CLEANUP:SPEC-01 -->
The existing PROGRAM_SPEC entry now maps nine governed formal domain owners,
links controlling product/tool authority, and distinguishes requirements status
from delivered behavior, qualification and execution permission. README links
that entry rather than introducing another master specification. Integration
promotion, completion requirements and structured/current status each retain
one named existing owner. No product requirement or grant changes.

<!-- EVIDENCE:SPEC-WORKSPACE-CLEANUP:SPEC-02 -->
Moved 213 lines of already-historical program v1, milestone criteria and user
stories into classified review history. Every moved section is byte-identical;
all original gate names, thresholds, standards boundaries and record context
remain. Existing root headings now link to those records, preserving inbound
anchors. Current ownership/scope/standards requirements remain in PROGRAM_SPEC.
The frozen coverage reader follows the historical tables without introducing
new current acceptance or changing recorded status. Coverage, two reader
regressions, governance and traceability pass. Current foundation/S5A approval
records remain pinned and unchanged; this bounded move does not rewrite them.

<!-- EVIDENCE:SPEC-WORKSPACE-CLEANUP:SPEC-03 -->
PROGRAM_SPEC and INTEGRATED_PROGRAM_SPEC now follow the existing controlling
operational/doctrine/tool-contract authority; directory placement confers no
precedence. The original authoring sequence is explicitly historical and
PLAN.md cannot select work. Repeated status/selector descriptions point to
one named section. Existing integration, defer/freeze and acceptance clauses
remain; executable identities are unchanged and current commands use the
existing guarded runner/companion prebuild. Classification distinguishes
governed clauses from unfinished full-spec promotion. Coverage, parity,
governance, traceability and Frontier validation pass without a Cargo rerun.
No product rule, acceptance threshold or original S5A grant changes.

<!-- EVIDENCE:SPEC-WORKSPACE-CLEANUP:SPEC-04 -->
Audited sixty target fixture collections plus 6 .datum native/reference-audit
collections against all tracked file bytes for
relative paths, absolute paths and collection names. Retain thirty-seven
referenced collections, including the 793 MiB/81-project current S5A inventory,
and both unreferenced user-edit collections. Twenty-one unreferenced generated
collections (230 project markers, 190.6 MiB) are preserved outside the checkout
at /home/bfadmin/Documents/datum-eda-workspaces/retired-fixtures-20261014.

Under the Cargo lock and host active-output checks, atomic same-filesystem
renames preserved every file hash, size and mode. No native project, journal,
revision blob or user data was deleted. External manifests and relocation
records plus ignored .datum/history-cleanup/fixture-retirement-map.json retain
recovery/dispositions. Progress receipts are persisted per collection. Existing
referenced paths stay valid; no replay, qualification or S5A resumption is
claimed. Independent pre-mutation review passed the exact preservation plan.

<!-- EVIDENCE:SPEC-WORKSPACE-CLEANUP:SPEC-05 -->
The S5A fixture generator now defaults to an external managed workspace with
task/session provenance, a stable owner/family path and an exclusive ownership
receipt. It refuses repeated reservations, checkout paths, unsafe labels and
symlink paths resolving into the checkout. Explicit roots remain available for
pinned historical replay. README examples create user projects outside source
and explain release, reference checks and preservation before retirement.

All nine fixture-generator tests pass, including actual native construction.
The three workspace tests passed again after the non-UTF8 path correction.
Actual command-line checks passed for omitted-root managed placement, repeated
reservation refusal with unchanged file hashes, and explicit-path compatibility.
Independent source and preservation review passed. These checks qualify this
bounded generator change, not S5A native acceptance or full-workspace health.

Implementation landed in `2bff5ff1`. Independent review passed the specification,
preservation, generator and fulfilled proof conditions. Final released-build
pruning removed eleven entries, retaining 568 native project roots and runtime
binaries; all .datum output and recovery metadata remain. Checkout allocation
is approximately 3.2 GiB. All five bounded maintenance requirements are complete.
Original S5A-C04 selection and its entire retained task/grant record are restored
without a claim; both working sessions remain paused. Foundation/S5A approvals
and protected prototypes are unchanged. No product, dependency, successor or
qualification grant is added. Existing dat-srf4, dat-wq2s, dat-afp9 and dat-lk0l
qualification failures remain open; no full-workspace PASS is claimed.

## Additional owner-directed cleanup

The owner explicitly selected these five improvements on 2026-10-14.
S5A sessions remain paused; product, dependency and protected prototype
authority remain controlling. Independent review is required before closure.

<!-- REQ:ADDITIONAL-CLEANUP:CLEAN-01 -->
1. Prune released build output preserving native projects and runtime binaries.
<!-- REQ:ADDITIONAL-CLEANUP:CLEAN-02 -->
2. Move history rollback backup outside the working directory preserving recovery.
<!-- REQ:ADDITIONAL-CLEANUP:CLEAN-03 -->
3. Enforce retention for temporary output and tracker backups.
<!-- REQ:ADDITIONAL-CLEANUP:CLEAN-04 -->
4. Compact redundant large evidence JSON preserving findings and source references.
<!-- REQ:ADDITIONAL-CLEANUP:CLEAN-05 -->
5. Extract cohesive registered source debt and verify behavior.

<!-- EVIDENCE:ADDITIONAL-CLEANUP:CLEAN-01 -->
Released-output preview and host process inspection found no active build/app.
The existing guarded pruner removed nineteen released output entries, reducing
target from 14 GiB to 3.0 GiB. It retained 795 native project roots, runtime
binaries and all rollback metadata. No tracked files were removed. Any caches
regenerated for the final source proof will be pruned again after that build.

<!-- EVIDENCE:ADDITIONAL-CLEANUP:CLEAN-02 -->
Moved the original history rollback mirror by atomic same-filesystem rename to
/home/bfadmin/Documents/datum-eda-backups/original-history-20261014.git. All
5,015 file hashes, sizes and modes match before/after; full Git fsck passes on
both locations. The preserved mirror contains 1,249,157,198 file bytes. Ignored
.datum/history-cleanup/rollback-location.json records its path and manifest hash.
No rollback data was deleted and no published history changed. The repository
working directory no longer carries that 1.2 GiB backup.

<!-- EVIDENCE:ADDITIONAL-CLEANUP:CLEAN-03 -->
The enabled repository post-commit hook prunes tracker recovery to the five
newest backups using br's own history command. Pruning eleven released backups
reduced history from 13 MiB to 5 MiB; canonical issues remain unchanged. The
existing development pruner now offers build-only retention of all .datum task
output and keeps runtime companion .d records to avoid unnecessary rebuilds.
Eight safety regressions pass, including protected output-root preservation
and removal of stale dependency records for alternate runtime copies.
An actual hook smoke test retains exactly five backups without changing
canonical issue bytes.
README gives preview/apply and task-release cleanup commands. Temporary output
is retired only after release; native projects and rollback metadata remain
protected. Artifact gates continue to forbid temporary captures in commits.

<!-- EVIDENCE:ADDITIONAL-CLEANUP:CLEAN-04 -->
Compacted six historical resize reports referenced only by the frozen capture
SHA256SUMS inventory: baseline-traced, baseline-plain, candidate-reuse-plain,
baseline-plain-v2, reuse-only-pilot and candidate-plain-idle. None is an owning
traceability consumer or current acceptance reference. Retired 4,852 raw
CPU/RSS/DRM counter samples, saving 5.52 MiB. Every preexisting non-raw parsed
field is identical; peak CPU values calculated by the original summarizer's
formula are retained with sample counts, array hashes and observed intervals.
Original commit/file hashes match the preserved historical SHA256SUMS pins.
Each report states raw data is unavailable and supplies no new qualification.
Original producers/summarizer continue to operate on temporary raw output.

The index artifact gate rejects raw counter arrays in new or changed reports;
unchanged source-pinned historical receipts remain intact. Six artifact tests
pass, including forced-add rejection, historical preservation, changed-record
refusal and nonmutation. Source health and traceability pass. Referenced final,
comparison and terminal records are unchanged.

<!-- EVIDENCE:ADDITIONAL-CLEANUP:CLEAN-05 -->
Moved six complete native orthogonal-graph reused-via artifact export/apply
cases into their cohesive 636-line child test module. Test bodies are
byte-identical, shared fixture owners and production commands are unchanged,
and the parent module/filter identity is retained. The largest registered test
file drops from 5,434 to 4,804 lines, with its exact ledger and parity ceiling
ratcheted downward. Remaining legacy debt stays open; no new oversized child,
exclusion or exception is introduced.

Source health, thirteen policy-integrity regressions, focused repository gates,
Cargo resource/dependency authority and independent source review pass. The
focused six-case guarded suite must finish before closure. Prior full drift's
sixteen unchanged engine Clippy failures (dat-lk0l) and other qualification
issues remain separate; no full-workspace PASS or weakened gate is claimed.

All five additional improvements are delivered in independently reviewed
commits 1fe0c31a and 78f23571. After the six-case source proof, the build-only
pruner removed nine regenerated output entries and returned target to 3.0 GiB;
.datum is 191 MiB, current Git storage 56 MiB and tracker backups 5 MiB. All
795 native project roots and runtime binaries remain; the earlier pruner
summary included one rollback directory in its 796-root count, now corrected.
The external 1.2 GiB rollback mirror remains intact and outside ordinary pruning.

The complete original UVT-S5A-BUILD task/grant is restored with no claim and
both S5A sessions remain paused. No product change, qualification PASS,
resumption or successor authorization is inferred. The four prior qualification
issues remain separately open. Released proof output stays local; tracker
recovery is automatically bounded at commits and cleanup is ownership-aware.

## Owner-directed organization cleanup

The owner authorized these five improvements on 2026-10-14. Product contracts,
required acceptance evidence and dependency authority remain controlling; the
S5A implementation sessions stay paused. Changes land in independently reviewed
units with concrete before/after evidence and bounded verification.

<!-- REQ:REPOSITORY-ORGANIZATION:ORG-01 -->
1. Reconcile README capabilities and guarded development commands; make current
   authority and historical plan roles clear through the existing entry points.
<!-- REQ:REPOSITORY-ORGANIZATION:ORG-02 -->
2. Compact unnecessary raw review JSON after reviewing consumers; preserve
   findings, method/source pins, acceptance records and truthful raw availability.
<!-- REQ:REPOSITORY-ORGANIZATION:ORG-03 -->
3. Group GUI application source and associated tests by cohesive responsibility,
   update active consumers and verify behavior without changing product semantics.
<!-- REQ:REPOSITORY-ORGANIZATION:ORG-04 -->
4. Remove actual repeated verification and provide clearly scoped focused/full
   commands while preserving required qualification and guarded Cargo execution.
<!-- REQ:REPOSITORY-ORGANIZATION:ORG-05 -->
5. Extract cohesive ownership from registered GUI protocol source debt, ratchet
   exact ceilings downward and verify retained behavior and existing source policy.

<!-- EVIDENCE:REPOSITORY-ORGANIZATION:ORG-01 -->
README now describes the delivered bounded library GUI workflow and separate
qualification, uses the pinned Rust toolchain and guarded Cargo examples, and
identifies current authority, review records and historical context. PLAN.md is
explicitly historical and classified accordingly. Existing product contracts
and acceptance records remain unchanged. Parity, coverage, governance and
traceability checks pass; these documentation changes require no Cargo rerun.

<!-- EVIDENCE:REPOSITORY-ORGANIZATION:ORG-02 -->
Compacted five unreferenced historical failed/partial GPU observer reports,
retiring 2,485 raw per-process/fd samples and saving 15.39 MiB. Every other parsed
field remains identical, including outcomes/errors, CPU and DRM aggregates,
method/source/binary pins and qualification limits. Each record names its
original commit/hash and explicitly states raw samples are unavailable; no new
qualification or acceptance is claimed. Producers retain their original methods
and write raw data to temporary output. Current acceptance maps/records remain
unchanged. The existing artifact gate rejects raw DRM arrays hidden inside
review JSON; five forced-index/retirement tests and source-health checks pass.

<!-- EVIDENCE:REPOSITORY-ORGANIZATION:ORG-03 -->
Rehomed 274 GUI source/support files into six responsibility directories; the
source root now contains only main.rs. Existing Rust module identities are
preserved with explicit path registrations. Main shrank from 693 to 315 lines;
terminal control/session/clipboard and shared tool actions are cohesive owners
of 308 and 188 lines. Associated tests remain beside their owners. Active source
consumers follow the new paths and continue recursive enforcement. Shared source
pattern matching moved into its own implementation; its legacy caller ratchets
from 987 to 968 lines. No production source ceiling or exclusion increased.

The broad guarded GUI run passed 447 tests with 33 explicit native/qualification
skips; four companion freshness tests timed out because nested builds waited
on the outer Cargo lock. Prebuilding matching companions removes that cycle;
five focused lifetime tests then pass in under one second. The remaining native
project-open case passes with host access (sandbox device/socket limitations).
Protocol tests passed 149 with three explicit skips; the one stale historical
pilot path comparison is reconciled through exact relocations without modifying
the retired receipt, and its focused registry tests pass. One unchanged
accessibility description assertion remains captured as dat-srf4. Existing
focus-initializer and stale journal-owner gates are captured as dat-wq2s and
dat-afp9; exact old/new source comparisons establish those inputs are unchanged.
Interaction, split, convergence, transport and workflow guard regressions pass;
source health, dependency authority, Cargo policy and traceability pass. These
results do not supply broader S5A/native qualification. ORG-04 addresses the
runner's companion ordering and nested-lock failure mode.

<!-- EVIDENCE:REPOSITORY-ORGANIZATION:ORG-04 -->
The default/explicit full drift command retains its original qualification
schedule, removing only the repeated MCP self-test already owned by alignment.
The explicit focused mode runs structural gates without Cargo and cannot claim
full qualification. Workspace health builds matching CLI/daemon companions
before tests and stops on setup failure. Nested Cargo guards refuse immediately
before locking, preserving serialization and ending the observed 30-minute
self-wait. CI socket smoke now uses the same guard. README names these scopes.

Eleven resource-guard tests and four orchestration regressions pass, including a
real nested child against a held temporary lock, default/full schedule equality,
retained native/migration gates and companion-failure short-circuit. Actual
focused gates and thirteen source-health regressions pass. Full drift was run;
Clippy fails on sixteen unchanged engine lints under the pinned toolchain,
recorded as dat-lk0l. Warning policy and all downstream qualification gates stay
intact; no full-workspace PASS is claimed. Exact source-health parity reflects
the ORG-03 downward ratchet. Independent source review passes.

<!-- EVIDENCE:REPOSITORY-ORGANIZATION:ORG-05 -->
Eleven board-scene geometry/display payload types and their existing outline
layer default now belong to the normal scene_primitives module (149 lines).
All declaration/default bodies are byte-identical; explicit root public exports
retain consumer API and Serde schema/defaults. Root production lines decrease
from 3,078 to 2,940, exactly ratcheted in the debt ledger and parity inventory.
The 1,705 existing inline test lines and remaining legacy decomposition debt
stay open; no exception, ceiling increase or invented qualification is added.

Source health, thirteen policy-integrity regressions, parity, dependency
authority and evidence traceability pass. Existing fixture round-trip, absent
field/default, materialization, geometry and authoring tests are reused.
Full drift's unchanged engine Clippy failures remain recorded as dat-lk0l;
repeating that unchanged failed build provides no additional decision evidence.
All 150 protocol behavior tests pass with three explicit qualification skips;
GUI consumer compilation passes. Independent source review passes the unchanged
types, exports/defaults and exact ratchet. These results complete ORG-05 without
claiming the separately blocked full-workspace qualification.

All five organization improvements are delivered in independently reviewed
commits 8636b8c3, fe1777d5, cb80fd51, 91fe7bc5 and 601fd644. Temporary raw output
stays ignored; screenshots and archives remain blocked from ordinary commits.
The existing guarded/cache-maintenance mechanism controls regenerated local
build output. No product contract, dependency or protected prototype changed.

The pre-maintenance UVT-S5A-BUILD/S5A-C04 pointer is restored with its entire
original task/grant byte-equivalent at the parsed-record level and no claim.
Both implementation sessions remain paused. No S5A acceptance, resumption or
successor authorization is inferred. Full qualification remains open separately
for dat-srf4, dat-wq2s, dat-afp9 and dat-lk0l; focused cleanup proof is recorded
above and never presented as full-workspace PASS.

## Owner-directed maintenance completion

The owner selected this bounded five-priority maintenance goal on 2026-10-14.
It preserves product requirements and leaves the other implementation sessions
paused. The owner explicitly approved the prepared history replacement; its application
and final consistency checks are recorded below.

<!-- REQ:REPOSITORY-MAINTENANCE:MAINT-01 -->
1. Repair legacy pilot tests using temporary self-contained fixtures.
<!-- REQ:REPOSITORY-MAINTENANCE:MAINT-02 -->
2. Reconcile specification parity and historical progress checks without
   weakening their requirements or changing product status.
<!-- REQ:REPOSITORY-MAINTENANCE:MAINT-03 -->
3. Retire redundant tracked raw proof while preserving useful text findings.
<!-- REQ:REPOSITORY-MAINTENANCE:MAINT-04 -->
4. Prune released local output and retired backups after ownership/use checks.
<!-- REQ:REPOSITORY-MAINTENANCE:MAINT-05 -->
5. Prepare and verify a compact history replacement, commit map, rollback backup
   and collaborator instructions.
<!-- REQ:REPOSITORY-MAINTENANCE:MAINT-06 -->
<!-- OWNER:REPOSITORY-MAINTENANCE:MAINT-06:HISTORY-REWRITE -->
6. Obtain approval for the exact prepared published-history replacement, apply
   it with an expected-old-head check, and verify remote/local consistency.

<!-- EVIDENCE:REPOSITORY-MAINTENANCE:MAINT-06-APPROVAL -->
The owner explicitly replied `approve prepared history rewrite` on 2026-10-14.
This approves the sealed replacement of main
`9adf84b3da33a8927bd218f41e4ab581f5526c85` with
`aefd8f2a30fead730dde4730527531d7e782a6d5`, its forty listed source-history tags,
and the listed retired local ref/reflog cleanup and repack. The original mirror
remains local for rollback. The approval packet and exact response are retained
in ignored `.datum/history-cleanup/` storage. MAINT-06 now executes that approved
transport and consistency check; no successor or S5A execution is authorized.

<!-- EVIDENCE:REPOSITORY-MAINTENANCE:MAINT-06 -->
Applied the approved main replacement atomically with the exact expected-old-head
lease and created all forty source-history tags. GitHub refs exactly matched the
sealed packet before local installation. Retired the 247 listed local refs and
old reflogs, repacked and passed Git integrity checks. Local object storage is
48.95 MiB. All 2,744 mapped source targets remain reachable; all 51 protected
prototype hashes and the four owned governance drafts were preserved.

A fresh full clone from GitHub received the replacement and all forty tags, with
all mapped targets reachable and Frontier/traceability checks passing. Its
transfer pack is 57.02 MiB (pack compression differs from the local repack).
All 9,269 reachable historical changed paths satisfy the artifact policy. The
old rewritten HEAD is absent locally and retained in the untouched original
1.2 GiB rollback mirror. No Cargo archive directories remain, including the
previous 66 GiB archive. Repository parity, coverage, governance, traceability,
Frontier and temporary-artifact gates pass without another Cargo proof build.
Independent source review passed the exact local installation procedure;
independent application and closure review also passed the actual local/remote
refs, fresh clone, historical targets, preserved backup and paused S5A boundary.

All five maintenance priorities are delivered. The pre-maintenance roadmap pointer UVT-S5A-BUILD/S5A-C04 is restored because
the selector requires exactly one canonical task. Its original state and grant
are unchanged, it has no claim, and both implementation sessions remain paused.
No S5A acceptance, resumption, new successor grant or broader qualification is inferred. Other sessions
must use the new history before committing; never merge/push the old history.

<!-- EVIDENCE:REPOSITORY-MAINTENANCE:MAINT-01 -->
Pilot capture unit tests now generate temporary extraction/accounting fixtures;
all eleven tests pass without the retired archive. This is transport unit proof,
not native pilot qualification or reinstatement.

<!-- EVIDENCE:REPOSITORY-MAINTENANCE:MAINT-02 -->
The supervision inventory follows five production owners and excludes test
decoys; its fifteen current markers are individually source-reviewed. Historical
M3 coverage checks preserve its 28 retained criteria and frozen execution boundary.
Two positive/negative reader regression tests, specification parity and progress
coverage checks pass. Product requirements/status are unchanged.

<!-- EVIDENCE:REPOSITORY-MAINTENANCE:MAINT-03 -->
Retired 783 raw logs/stdout/stderr/JSONL files and 912 raw accessibility/window/
legacy state checkpoints (24.45 MiB). Existing JSON/Markdown findings, reviewed
method sources and required state/qualification records remain. None of the
removed paths is a governing/traceability/current completion reference. Ignore
rules and the forced-add gate reject these raw output formats. Historical raw
references no longer promise current replay availability.

<!-- EVIDENCE:REPOSITORY-MAINTENANCE:MAINT-04 -->
Removed 7,603 released local output entries while preserving 773 native project
roots, including projects inside old source snapshots, and one hash-verified
current runtime binary set. Target allocation fell from about 32 GiB to 3.0 GiB;
.datum from 1.3 GiB to 188 MiB. Retired WDQ storage (512 MiB) was removed;
tracker backup retention was reduced from 98 MiB to 5 MiB (five backups).
The known clean detached historical worktree was separately removed.

The prune tool previews every deletion, holds the Cargo lock, refuses tracked
output and active builds/apps, preserves project trees, and validates runtime
copy paths before any mutation. Process inspection has a stated limitation:
four nondumpable installed desktop/security services (SSH agent, KWin, KDE
polkit agent and power manager) prevent full open-file inspection and are
reported as limited visibility. Inspectable processes and children are checked.
Four orphaned engine daemons had no connected clients and were stopped first.
Six cleanup safety regression tests pass. Independent source review passed the
first four priorities; it does not cover history replacement or S5A acceptance.

<!-- EVIDENCE:REPOSITORY-MAINTENANCE:MAINT-05 -->
Initial preparation from source `6bb0fa7e` verified all 2,743 commits: retained
file bytes/modes, author/committer, message and mapped parent order are unchanged.
The finalized candidate adds only the historical locator map to the current
source tree. Forty source-history tags keep every mapped commit reachable,
including all five required non-main source pins. The clean pack is 49.18 MiB;
the untouched original mirror is about 1.2 GiB and stays local for rollback.
Independent preparation review passed without publication authorization.

A fresh transport clone has no old changed HEAD object; Frontier, parity,
progress coverage, governance, traceability, artifact and source-health checks
pass, along with 57 locator/delivery/Frontier tests. No Cargo rerun was needed:
retained Rust and protected prototype bytes are unchanged. Final preparation
must include this governance handoff and repeat transport/metadata verification
before the exact approval packet is presented.

The sealed exact packet is `.datum/history-cleanup/approval-packet.json`;
`source-history-tags.json` lists every proposed new published tag and target.
Publishing replaces only main and creates those tags atomically, with an exact
expected-old-main lease and refusal of tag collisions. Local installation then
removes retired old custom refs/reflogs and repacks; the original mirror remains
untouched for rollback. The exact replacement was explicitly owner-approved and applied as recorded below.

Collaborators should use a fresh clone after publication, fetch
`refs/tags/datum-source-history/*`, and preserve any local edits separately before
moving them to the new clone. Merging or pushing old history would restore the
removed archives. The paused implementation sessions receive no new execution
grant from this maintenance. Rollback uses the untouched original mirror and
an exact expected-new-main lease; published tags must be reconciled explicitly.

History preparation keeps the original repository and refs in ignored
`.datum/history-cleanup/` storage, separate from the proposed replacement.
Historical evidence IDs stay unchanged. The optional `specs/history_commit_map.json`
locates retained source under its new ID only when the mapping is unique and the
target commit exists. It does not establish acceptance or restore removed raw
proof. Candidate validation must prove unchanged retained file bytes/modes,
author/committer/message and mapped parents before publishing this map. Required
historical source outside main must remain reachable through explicit source
history tags; collaborators must fetch those tags with the replacement.

The owner's 2026-10-13 direction is controlling: screenshots used to demonstrate
work are temporary. Do not commit or push them, recordings, raw capture archives,
or compressed diagnostic bundles. Raw per-process DRM samples also belong in
temporary output; retained JSON findings use explicit summaries and availability.
The pre-commit temporary-artifact gate enforces
this rule, including forced additions. It also rejects compressed archives
anywhere in the repository so moving a bundle into testdata cannot evade the rule.

Write temporary proof to an ignored, task-owned directory under `target/` or to
an external temporary directory. Keep it only while needed for debugging or
review, then remove it. Before removing local output, check ownership and active
processes; never delete another session's live work or clean active Cargo targets.
Do not preserve duplicate build trees indefinitely as proof archives.

## Cargo cache maintenance

Run `python3 scripts/manage_cargo_cache.py` for a read-only usage report.
It separately lists large direct proof logs; these are temporary verification
output rather than Cargo caches and need task-owner cleanup after review. Do not
run unbounded diagnostic tracing or preserve multi-gigabyte stdout indefinitely.
Use the shared `target/` for ordinary builds. A separate task-owned build tree
must be released and removed when its build/review use ends; do not move it into
`.datum/` to retain it indefinitely. Preserve needed findings as text instead.

For a reported disposable Cargo target, first preview the exact path:

```sh
python3 scripts/manage_cargo_cache.py --remove target/completed-proof
```

After confirming the task has released it, append `--apply` to remove it. Multiple
explicit `--remove` paths are supported. This is a local command, not a scheduled
automatic eviction. It holds the existing Cargo resource lock and refuses while
Cargo, rustc, the GUI or engine daemon is active. It protects shared `target/`,
`debug/`, `release/`, tracked files, unmarked fixture/evidence directories, unknown
directories and symlinks. A separate target with extra non-Cargo root entries
also refuses cleanup. Selecting an eligible cache removes its entire tree,
including any output or fixture placed inside its build subdirectories; inspect
the contents and release ownership before applying cleanup. A running executable
inside any selected cache refuses cleanup even if its process name is unfamiliar.
Only separate Cargo trees with Cargo markers, named historical
`.datum/agent-cargo-archive-*` directories, and explicitly selected shared
`target/debug/incremental` or `target/release/incremental` caches are eligible.
Clearing an incremental cache preserves binaries but makes the next incremental
build redo that cached work.

The Cargo guard now applies its existing soft/hard storage limits to the combined
`target/`, named historical Cargo archives, and any external selected target,
without counting a nested target twice. Choosing another small target no longer
bypasses the retained-cache limit. Limits are unchanged; cleanup is explicit and
never initiated by a build or by an elapsed-time guess about another task.

Commit concise text findings, relevant source revisions, reproducible commands,
review disposition and limitations. Hashes can identify artifacts that existed
during review; they do not promise permanent artifact availability. Screenshots
alone do not establish acceptance, and this storage policy does not waive required
runtime checks or independent review.

Automated image comparison inputs under `crates/gui-render/testdata/golden/` and
the two owner-approved design references under `docs/gui/reference/` remain test
and design inputs. Incidental `revision/running-app/` captures are excluded.
Authored product images belong under `crates/**/assets/`; authored diagrams are
also distinct from temporary work proof. Do not relocate screenshots into these
input directories to bypass the policy. Other raster images and recordings are
ignored and rejected by the gate even in newly named proof directories.

## Bounded GUI diagnostics

`DATUM_ACTION_EVIDENCE=1` now emits `datum.action-evidence/v2`. Selection records
contain kind, revision, scope, focus, counts, exact sorted membership SHA-256
digests and at most eight identity samples. They omit complete property and
projection maps and explicitly set `complete_inspection=false`. Normal launches
leave this diagnostic stream disabled. There is no full-Debug-dump override.

The stream limits a record to 64 KiB and a session to 8 MiB, reserving space for
an explicit `diagnostics_incomplete` event before stopping output. These are
diagnostic storage limits, not selection limits or product performance budgets.
They provide ample room for the ordinary registry and compact state, while
preventing a few interactions from generating gigabytes. Stream exhaustion is
never a successful or complete proof receipt. Other timing/verbose output is
separate and should be enabled only for the specific diagnostic question.
The byte bounds do not qualify diagnostic memory use or input latency: exact
identity hashing still scales with membership and uses temporary serialized
buffers, and the existing asynchronous queue is not a bounded-memory queue.

Legacy pilot observations reject v2 and incomplete records because the summaries
do not satisfy their old complete-selection assertions. Historical, hash-pinned
scratch readers need deliberate migration before use with current binaries; do
not silently substitute summaries for those assertions. Exact field/membership
checks should query ordinary typed context/accessibility/engine interfaces at the
required checkpoints rather than repeatedly serialize internal state to stderr.

The owner-directed 2026-10-14 cleanup purged 31 obsolete S5A stdout logs and
duplicate full-state excerpts, reclaiming 8.09 GiB after confirming no application,
build or open log reader/writer was active. Their old paths and hashes describe
historical observations, not currently available replay input. Existing derived
findings remain at their recorded scope; this cleanup does not claim S5A passes.

## Historical records

This cleanup removes accumulated proof images and compressed bundles from the
current tree. Older review records retain their original observations, paths and
checksums as historical records. Those paths no longer guarantee files in a fresh
checkout. Do not claim a historical bundle was replayed when it is unavailable;
regenerate only the evidence actually needed for a current decision.

The retired manual `scripts/workflow_delivery_pilot_capture.py` and
`scripts/workflow_delivery_pilot_package.py` require the removed historical
`research/process-quality/evidence/wdq-c01-current/native-fixture.tar.gz`.
They cannot replay that pilot from the cleaned checkout. Historical diagnosis
requires the pre-cleanup tree; this does not reinstall retired WDQ enforcement
or authorize a historical pilot replay as a prerequisite to current work.

The pre-cleanup tree at `cfd24461` contains the historical files. Git history has
not been rewritten, so historical objects still contribute to clone size. A
separately authorized history cleanup is required to remove those remote objects.
This change does not complete S5A or change its acceptance requirements.
