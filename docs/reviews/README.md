# Review records and temporary proof output

## Documentation ownership and external audit output

Owner selected the three organization improvements on 2026-10-14. Preserve
current requirements, unique rationale, retained inputs and paused S5A.

<!-- REQ:DOCUMENTATION-OWNERSHIP-ORGANIZATION:MAP-01 -->
- MAP-01: Clarify authoritative documentation ownership and navigation without adding rival product rules.
<!-- REQ:DOCUMENTATION-OWNERSHIP-ORGANIZATION:MAP-02 -->
- MAP-02: Consolidate verified overlapping requirements into their existing authoritative homes while retaining unique rationale and historical recovery.
<!-- REQ:DOCUMENTATION-OWNERSHIP-ORGANIZATION:MAP-03 -->
- MAP-03: Move remaining checkout-level .datum test producers to external workspaces, verify, independently review and publish all three improvements.

<!-- EVIDENCE:DOCUMENTATION-OWNERSHIP-ORGANIZATION:MAP-01 -->
Added the [documentation map](../README.md), linked from root README. It routes
questions to existing doctrine, domain contracts, formal subject owners,
canonical IR, selected status/grant, review evidence, research and source/test
owners. It explicitly preserves CANONICAL_IR's governed ownership and separates
ENGINE_DESIGN's rationale. Current/target/compatibility/historical labels and
requirement versus delivery versus qualification remain distinct. It introduces
no product rule or competing roadmap. The map is classified and this selected
maintenance task supplies its roadmap placement; no new inventory shapes.

<!-- EVIDENCE:DOCUMENTATION-OWNERSHIP-ORGANIZATION:MAP-02 -->
Consolidated the verified engine/IR overlap into existing owner links.
ENGINE_DESIGN no longer carries parallel Pool/Board/Schematic field diagrams,
dynamic Operation trait or Engine/execute_batch method declarations. It links
exact engine type/status, operation/batch/commit, API and native persistence
owners while retaining headless rationale, pool inspiration/index rationale,
workflow-family illustration and rule examples with explicit nonnormative scope.
The unrestricted "read any version >=1" claim in CANONICAL_IR now delegates to
NATIVE_FORMAT_SPEC §8's supported-version/future-refusal/current-target contract.
No formal engine/native contract or product decision changed.

CANONICAL_IR retains governed ownership. Its complete PM053/PM054 reconciliation
suffix is byte-identical to 62819c9b, as are its recursive RuleScope/JSON,
coordinate conventions, index rationale, integer precision and invalidation
motivation. Both edited documents have no owning evidence route; existing route
sources/consumers/digests are unchanged. Governance classification and this
Frontier entry identify their roles without promoting nonnormative examples.
All replaced wording remains retrievable with `git show 62819c9b:<original-path>`.
Independent assessment identified these boundaries before editing; independent
final implementation review passed. This resolves these concrete overlapping definitions, not every
remaining semantic duplication candidate in the broader intake.

<!-- EVIDENCE:DOCUMENTATION-OWNERSHIP-ORGANIZATION:MAP-03 -->
Eleven existing CLI audit fixture producers now call the shared development
workspace owner, registered once in the CLI test parent. New roots go to
`$HOME/Documents/datum-eda-workspaces/scratch/footprint-reference-audit/`
or the explicit external `DATUM_DEVELOPMENT_WORKSPACE` base, keeping each
existing UUID-qualified fixture name. Fixture payloads, authored operations,
refusal assertions and history/reopen tests are unchanged. A project's own
`.datum/journal` remains inside that project; only checkout-level placement
changed. Existing local audit inputs and recovery controls are preserved.

Verification uses the existing 32 affected tests only, in one guarded CLI test
run with a task-owned external base and disk-backed temporary directory. This
requires a cold test-binary compilation after prior cache pruning. Source-health
policy regressions pass (13 tests), and the focused drift battery passes:
resource, dependency, preferences, terminal boundaries, design tokens, governance,
parity, traceability, progress and Frontier checks. All 63 relative links/anchors in the four edited navigation/architecture
documents resolve. All 32 affected CLI tests passed in one run (28.75 seconds of test execution,
908 unrelated cases filtered). The external audit family was created and empty
afterward because the tests already remove their own projects. All 18,769
retained local audit metadata records (paths, modes, sizes and modification
times) match their pre-test snapshot. No retained fixture was moved or deleted.
The first release check incorrectly expected surviving project markers; it
refused without deleting output. Corrected it to require the empty external
family and unchanged retained inputs; no compilation or test rerun followed.
Removed that empty exclusive verification workspace after active-user checks.
The guarded build-only prune removed nine released compiler entries, preserving
all native inputs/recovery controls and the three current executables.

Independent final implementation review passed for all 21 owned paths; landing
closure and publication follow separately.
The eleven producers now use external scratch. Older cleanup records below
state the paths/producers observed at their dated revisions; this section owns
the current producer placement. Broad semantic-documentation reconciliation
remains open as unselected intake, without an automatic successor.
No GUI/performance campaign or full qualification is inferred.

## Organization and evidence cleanup

Owner selected the measured cleanup on 2026-10-14. Native data, current
product authority, source pins and approvals remain protected; S5A stays paused.

<!-- REQ:ORGANIZATION-EVIDENCE-CLEANUP:ORG-01 -->
- ORG-01: Reduce redundant tracked raw proof while retaining exact historical recovery, conclusions, approvals and source pins.
<!-- REQ:ORGANIZATION-EVIDENCE-CLEANUP:ORG-02 -->
- ORG-02: Classify local .datum attempts and preserve or retire them only with ownership, references and recovery verified.
<!-- REQ:ORGANIZATION-EVIDENCE-CLEANUP:ORG-03 -->
- ORG-03: Consolidate obsolete historical planning into existing documentation owners without changing current requirements.
<!-- REQ:ORGANIZATION-EVIDENCE-CLEANUP:ORG-04 -->
- ORG-04: Prune released Python caches and excess tracker backups; assess retained executables without rebuilding.
<!-- REQ:ORGANIZATION-EVIDENCE-CLEANUP:ORG-05 -->
- ORG-05: Verify, independently review and publish the organization and evidence cleanup; restore original paused S5A state.

<!-- EVIDENCE:ORGANIZATION-EVIDENCE-CLEANUP:ORG-01 -->
The combined infrastructure independent observation file duplicated all 1,123
objects in the six retained `infrastructure/independent/infra-s01-observations.json`
through `infra-s06-observations.json`. A complete canonical-object multiset
comparison, including duplicate multiplicity, matched exactly. Those scenario
records remain the owning scenario copies. The retired aggregate reported
`complete_matrix: true`, `acceptance_asserted: false`, reviewer
`wdq-independent-review-20260906`, source
`9cab72daab46122da6d2c45ab7f972814f702880`; these remain historical statements,
not a new review or product acceptance.

Also retired the raw `infrastructure/independent/archive-members.json` capture
name list: 12,788 capture names and 59 exact fixture roots. Its scope was fresh
raw observations, original recipe attempts and failed clone setup; exact
post-setup Git objects were fixture inputs. It asserted neither native EDA
qualification nor exhaustive descendant-runtime closure. No project or input
was deleted. Unique inventory details remain recoverable byte-for-byte below.
Recovering the inventory does not restore raw captures or establish qualification.
Both retired files had no current tracked full-path consumers; the historical
replay entrypoint and source-pinned records remain unchanged.

All paths in this recovery table are relative to
`docs/reviews/workflow-delivery-rollout/infrastructure/independent/`.
Retrieve with `git show 97749064:<full-original-path>`; verify the SHA-256.
The commit remains in main history; this is current-tree consolidation, not a
history rewrite or removal of old Git objects.

| Retired file | Bytes | SHA-256 |
| --- | ---: | --- |
| complete-observations.json | 1,878,669 | `0b7181a00829adb62a0d2fac80359c0d13fe198177548bd4d38906bfd36b1278` |
| archive-members.json | 1,372,656 | `cdbad5335adab72644a20a6a63008c56dc18ace1b9b7bb6516b2aebda443c5a9` |

Review-storage disposition: owner/reviewer receipts, source pins, contracts,
per-scenario observations and hashed input inventories remain tracked. Source-
pinned measurement/replay records are retained unchanged. Large file size or
missing references alone do not authorize deleting them. Future capture-member
lists, like raw counters, must stay external; the staged-artifact gate refuses
new or changed payloads while preserving unchanged historical receipts. Seven
isolated-index gate checks pass, including refusal without index mutation and
acceptance of concise summaries.

<!-- EVIDENCE:ORGANIZATION-EVIDENCE-CLEANUP:ORG-02 -->
Classified all remaining .datum collections. Preserved and atomically relocated
`effective-process-delivery` (seven project markers) and `library-gui-delivery`
(three markers), totaling 87,305,264 file bytes, to
`$HOME/Documents/datum-eda-workspaces/retained-datum-delivery-20261014/`.
Complete before/after manifests matched every file hash, mode and empty directory.
A host process check and the Cargo lock preceded movement; no native data was
deleted. Independent plan review passed. No live producer/consumer uses either
old collection; the isolated resolver safety test uses its own temporary root.

Historical delivery receipts and source pins remain unchanged. Resolve their
old `.datum/<collection>/<suffix>` paths with
`python3 scripts/development_workspace.py resolve <old-path>`. This uses a
separate `.datum/history-cleanup/development-relocation-map.json` (or explicit
`DATUM_DEVELOPMENT_RELOCATION_MAP` override), mirrored beside the preserved
collections with a sealed full-manifest plan. Only these two .datum families
are allowed; traversal, incomplete states and suffix aliases into checkout
refuse. Existing target recovery maps and native input bytes are unchanged.

`footprint-reference-audit` remains protected locally: 62 native project markers
and eleven current CLI test producers still write there. Its retained
accepted-delivery references are not deletion authority. The two library-link
admission audits, symbol-unit audit and proof-source archive also remain
preserved. `.datum/history-cleanup` stays in place as recovery control data.
Nine workspace safety tests pass, including separate-map lookup and refusal of
unapproved families and child symlink aliases. Relocation changes storage only;
it asserts no native qualification, acceptance or new execution authority.

<!-- EVIDENCE:ORGANIZATION-EVIDENCE-CLEANUP:ORG-03 -->
Consolidated three historical structural records into
[STABILIZATION_PLAN.md](../STABILIZATION_PLAN.md), retaining dated findings,
reported outcomes, the still-unfinished later phase and exact recovery. Removed
the decomposition plan/backlog from the current tree; their retired script,
PR-lane, numeric-budget and automatic-fallback instructions no longer appear
as executable guidance. Current PM022 and roadmap/grant authority are unchanged.

Retired `docs/IMPLEMENTATION_PLAN.md`: its M0-M2/import-first sequence, no-GUI-yet
instruction, old milestone next tasks and crate-ownership snapshot describe an
obsolete development stage. Engine/CLI/daemon/harness/MCP boundaries remain
owned by current engine/interface specifications and controlling doctrine;
no new implementation order or product requirement replaces this snapshot.
Retired `docs/PRODUCTION_CLEANUP_MANIFEST.md`: it classified the June 19 dirty
worktree, reported source/import/text cleanup and then-passing workspace,
drift and visual checks, and called for staging/review. Those are dated claims,
not current production sign-off. Source assets, visual goldens, legal material,
protected prototypes and approval records were not removed with that manifest.
Its pending governance entry is removed; the consolidated summary is historical.

Retrieve each complete original with `git show 97749064:<original-path>`:

| Original path | Bytes | SHA-256 |
| --- | ---: | --- |
| docs/DECOMPOSITION_PLAN.md | 5,591 | `3dfdf0ea753ae2112c6fdd967e8a6be79978e2051d2cb694ddd3df862ff1dc25` |
| docs/DECOMPOSITION_BACKLOG.md | 8,479 | `42a05ed2016df0a1c79846b79206dd079a9938338be7125981b984ee301168bb` |
| docs/STABILIZATION_PLAN.md | 13,354 | `f12334027c1ed0a1346060cd2a6e45c6ba4fe33b24080669694b302919df8c0b` |
| docs/IMPLEMENTATION_PLAN.md | 9,680 | `069fb86fec86d8aedb143d5e7f90c99c6952ce53e5deabbab365df771a616674` |
| docs/PRODUCTION_CLEANUP_MANIFEST.md | 7,336 | `d7332bcd82b42fb0d7b390dd5066f2cbc8b0974ad3673281d50918c32947b9ab` |

All five records have no owning evidence route; they are not sources of current
product semantics. Live executable readers do not reference these filenames.
References in PLAN's dated stabilization paragraph still resolve to the retained
summary. The explicitly historical architecture tree remains an original
snapshot, as with previously retired checklists/guardrails; it is not a current
file inventory. Broader requirement duplication and protected evidence routes
remain outside this bounded consolidation; no complete semantic-cleanup PASS.

<!-- EVIDENCE:ORGANIZATION-EVIDENCE-CLEANUP:ORG-04 -->
Removed eight verified ignored Python bytecode cache directories (258 .pyc
files, 3,242,460 file bytes) under scripts, MCP and old review tools. Each
contained only untracked regular .pyc files; source, owner scripts, native data
and mixed directories were excluded. Subsequent verification uses
`PYTHONDONTWRITEBYTECODE=1` so this cleanup does not immediately regrow them.
The existing post-commit `br history prune --keep 5` policy already bounds
tracker backups; ran that supported command and removed one excess backup.
The canonical issues export and local tracker database are retained. Python
caches can regenerate normally; no new expiry service or cleanup framework.

Assessed the retained executables with ELF section inspection: CLI 418,791,840,
GUI 741,866,136 and daemon 268,799,880 file bytes; combined debug sections are
1,204,763,587 bytes. They account for most remaining checkout storage. Retained
them unchanged so existing ordinary application/replay binaries and their
pins remain usable. Stripping changes binary hashes; deleting requires later
rebuilds. Neither is needed for this documentation/cache cleanup. No Cargo,
GUI measurement, full-workspace suite or historical proof campaign was run.

<!-- EVIDENCE:ORGANIZATION-EVIDENCE-CLEANUP:ORG-05 -->
Bounded implementation verification passed: seven isolated-index artifact gate
checks and nine workspace safety tests. These cover real Git-index refusal and
nonmutation, unchanged historical receipts, concise summaries, separate recovery
maps, traversal/partial-state/symlink refusals and preserved migration behavior.
Source health checked 3,166 files; Cargo resource policy, specification governance
(552 classified), parity (30 inventories), evidence traceability (29 routes,
131 research/prototype artifacts), progress coverage, generated projection and
Frontier (89 items) pass. No Rust source, dependency, product or prototype changes;
no Rust compilation or repeated historical proof campaign was needed.

Independent reviews passed for exact duplicate retirement, native preservation,
the scoped resolver/gate and historical documentation consolidation. Before
landing, current tracked regular files total 109,590,404 bytes (104.51 MiB),
including 57,241,687 bytes under reviews. There are 507 tracked text documents.
Checkout allocation is 1,747,828,736 bytes (1.63 GiB), down from the preceding
1.73 GiB snapshot; preserved native data still occupies external disk storage.
Target still has zero project markers and its unchanged development binaries.
No Python bytecode cache directories remain in the inspected source/review roots.
These are allocation snapshots, not clone size or global disk-space savings.
Implementation landed in `6a973551` after independent final review PASS for
all sixteen owned paths. All five organization/evidence steps are delivered;
the maintenance claim is released. Every pre-maintenance Frontier record is
restored exactly, including the original S5A grant/pointer without a claim or
resumption. Broader semantic consolidation stays open as unselected intake
`dat-documentation-semantic-reconciliation-olnn`; this closes only the bounded
cleanup reviewed above.

## Cargo output and native workspace separation

Owner selected all four tasks on 2026-10-14. Preserve native inputs, historical
approval pins and paused S5A; provenance does not create execution authority.

<!-- REQ:TARGET-WORKSPACE-SEPARATION:TARGET-01 -->
- TARGET-01: Reserve target for disposable Cargo output and prevent fixture producers from repopulating it.
<!-- REQ:TARGET-WORKSPACE-SEPARATION:TARGET-02 -->
- TARGET-02: Preserve and relocate retained fixture collections with hashes, recovery records and updated live readers.
<!-- REQ:TARGET-WORKSPACE-SEPARATION:TARGET-03 -->
- TARGET-03: Separate user-owned or unclassified project data from generated fixtures without guessing ownership from directory names.
<!-- REQ:TARGET-WORKSPACE-SEPARATION:TARGET-04 -->
- TARGET-04: Remove released temporary collections after reference, process and ownership review; verify, review and land all four tasks.

<!-- EVIDENCE:TARGET-WORKSPACE-SEPARATION:TARGET-01 -->
Cargo output now contains only the three retained development executables and
companion dependency records. Zero project.json markers remain under target.
Current default Rust fixture/capture producers, terminal proof defaults and measurement
scripts use external workspaces. Shared path helpers check final output paths,
including child symlink aliases; fixture generation rejects explicit Cargo
output roots. The guard rejects native project markers before building in the
shared or any selected target. Cargo binary paths stay unchanged.

<!-- EVIDENCE:TARGET-WORKSPACE-SEPARATION:TARGET-02 -->
The independently reviewed 39-collection plan preserved 31 fixture collections
(464 project.json markers) under $HOME/Documents/datum-eda-workspaces/retained.
All files, directory modes and empty directories matched complete manifests
across atomic same-filesystem moves. Live Rust readers resolve old paths and
setup-record fields through the verified local map; historical JSON/native
bytes, source pins and approval records are unchanged. Prepared intent and
manifest records were saved before each move. Final recovery has mirrored maps
and a sealed full-manifest plan outside the checkout, verified SHA256
'ebd9467df9e72e056af20c5325f794981ada0d45feef39cf450c7861b3cf44e6'.
Five representative existing CLI validation reports match completely modulo
relocated paths and retain exit 1/invalid-design issues; this is relocation
equivalence, not native qualification or S5A acceptance. The first sandboxed
external read could not acquire a normal lock; the authorized host comparison
established the unchanged results without repairing project data.

<!-- EVIDENCE:TARGET-WORKSPACE-SEPARATION:TARGET-03 -->
Two collections (four project.json markers), user-edit-placement and
user-edit-qa-live, had no tracked references and insufficient ownership evidence.
They are preserved separately under $HOME/Documents/datum-projects/
recovered-development-20261014 with the same complete manifest verification.
Their names do not establish human or test ownership. They remain protected
pending owner disposition. Other user-edit-named collections have proof/test
provenance and remain retained inputs; none was discarded based on its name.

<!-- EVIDENCE:TARGET-WORKSPACE-SEPARATION:TARGET-04 -->
Retired six explicitly released generated test collections after verifying
producer provenance, no remaining tracked references, no active users and exact
fresh manifests: `diagnostic-prepared-selection-tests`,
`open-project-actual-worker-tests`, `open-project-app-tests`,
`open-project-protocol-tests`, `proposal-refresh-tests` and
`run-prepared-selection-tests`. These contained forty temporary project.json
markers and 1,252,538 file bytes. All 33 retained/unclassified collections remain
present; all six retired collections and their old target paths are absent.
Four additional task-owned verification outputs (the exclusive temporary tree
and three PID-qualified copies from the blocked attempt) were released after
successful checks. The protected release record names their ownership/basis.
The guarded build-only prune removed ten released Cargo output entries while
retaining all .datum inputs/recovery metadata and runtime executables.

Verification: eight current Python workspace safety checks pass; nineteen
unchanged guard/pruner results are reused. Seven current Rust workspace tests
pass. Five distinct project-opening cases pass: two passed initially, and the
three missing-companion cases passed after the required matching prebuild;
failed attempts are not added to success counts. All changed engine, GUI and
CLI test consumers type-check. The final GUI/CLI check includes the feature
scope and PID-owned scratch correction, with only the preexisting unused
PanePlan::overflow warning in unchanged gui-render source. No GUI/performance
campaign or full-workspace qualification is inferred. Source health (3,166),
resource/dependency authority, specification governance, parity, traceability,
progress coverage and Frontier checks pass. Independent relocation and path-
safety reviews passed. Independent final implementation review passed for the
39 owned paths landed in b3920995; no repeated builds or fixture hash batches
were required. All four cleanup tasks are delivered. The maintenance claim is
released and the complete pre-maintenance S5A task/grant is restored; its
implementation sessions remain paused. Broader documentation reconciliation
remains open as intake dat-documentation-semantic-reconciliation-olnn.

After released-output cleanup, checkout allocation is 1,862,578,176 bytes
(1.73 GiB), target is 1,428,381,696 bytes (1.33 GiB), .datum is 222,769,152
bytes and Git metadata is 61,022,208 bytes. These are local allocation snapshots,
not clone size. Preserved project data still occupies external disk storage.
Older cleanup records below describe their own boundaries and locations;
this section and the verified relocation map own the current storage layout.

External development layout after the owner-directed separation:

- `$HOME/Documents/datum-eda-workspaces/retained/`: preserved generated fixture
  and replay collections. These are inputs, never Cargo caches or automatic
  deletion candidates.
- `$HOME/Documents/datum-projects/recovered-development-20261014/`: two
  collections whose ownership is unclassified. Their directory names do not
  prove user or test ownership; preserve them until the owner decides.
- `$HOME/Documents/datum-eda-workspaces/scratch/`: new test/capture output,
  separated by producer family. Retire only an explicitly released family or
  task after checking current references and active processes. Failure output
  is not automatically released and no age-based eviction is enabled.
- `target/`: Cargo output and retained development executables only. The Cargo
  guard refuses native project markers in shared or selected build targets.

`DATUM_DEVELOPMENT_WORKSPACE` selects an absolute external development base.
`python3 scripts/development_workspace.py scratch <family>` prints its scratch
location; `resolve target/<old-collection>/<path>` translates a preserved
historical path. Rust proof consumers share the same checked path policy.
The local relocation map is `.datum/history-cleanup/target-relocation-map.json`,
with a mirror and sealed full-manifest plan outside the checkout, plus file/directory manifests
in the protected history-cleanup control directory. An interrupted migration
must be inspected from its persisted prepared/preserved states, never blindly
rerun. Old evidence JSON, approval pins and native input bytes are not rewritten.
A missing/retired/incomplete map entry refuses rather than inventing a fixture.
The map provides path recovery, not a new proof or execution grant. Regeneration
or replay still needs its existing authorization and current prerequisites.

The S5A fixture generator retains owner-labelled external managed placement;
explicit fixture roots now also refuse Cargo target paths, including aliases.
External explicit replay paths remain supported. Cargo binary/dependency paths
are unchanged; golden test inputs, protected prototypes and product semantics
are untouched.

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

At the 8d0e8ca4 assessment boundary, target occupied 2.78 GiB allocated:
debug held 1.33 GiB in the retained CLI,
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

First consolidation landed in `bb9d15f7`: four files retired, 515 tracked text
documents reduced to 511, and duplicate capability prose replaced with existing
ownership links. Independent review passed after preserving historical M0
closure facts. Source health (3,161 sources), governance, evidence traceability,
parity, progress coverage and Frontier checks pass. No Cargo proof rerun was
needed or performed. Bounded DOC-A01–03 are complete; remaining semantic
reconciliation stays open and is not automatically scheduled. Original paused
S5A task/grant is restored exactly without ownership or execution resumption.

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

Write temporary proof to an external, task-owned development workspace. Reserve
`target/` for disposable Cargo output. Keep proof only while needed for debugging or
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
