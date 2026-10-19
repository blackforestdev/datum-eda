# Native project doorway — managed delivery

Status: NPD-C01..06 remains accepted at `45f77006c661019e02928b90dba2a9f1b177bc4d`;
C07 repair is accepted at `0a8e8b5db1547f4c8d62983f397db99cc4b76fbc`.
NPD-C08 is accepted at `db81f73442dc5f33c1d125a4e40de5b7485fca60`: real terminal before
Project Open, canonical CLI creation and preserved sessions across GUI adoption.
The C08 pinned QA cohort is available; complete qualification remains separate.
NPD-C09's shared New Folder/Rename controls have passed independent bounded
source/runtime/rendered and package QA review at `file-actions-f1336e25`. New/Open use the
same popup, editor, browser controller and host service; Units remain automatic
and configurable in Preferences. The manager accepts the available QA package;
the actual delivered launcher passed its native Wayland/WGPU smoke check.
The owner-approved terminal handoff correction below has passed independent
bounded source/runtime/rendered/package review and manager acceptance. The new
QA package is `terminal-handoff-ec2cd61b`; the accepted f133 package stays frozen.
The owner then confirmed panel preservation and reported the project.json
double-click gap. Its bounded activation correction below has passed independent
review and manager acceptance at `manifest-open-94d59290`; ec2 stays frozen.
Owner QA then confirmed manifest activation and requested partial-path filtering
and keyboard completion. That shared Location refinement below has passed
independent bounded review and manager acceptance at `path-assistance-e12d125d`;
the accepted 94d package remains frozen.
Owner follow-up QA reports successive `datum-ed`/`datum-eda` completion failure.
The shared edit/navigation boundary correction below has passed independent
bounded review and manager acceptance at `successive-dee25e1e`; the e12d
package and its original bounded receipts remain frozen.
Owner follow-up QA now reports hidden initial Home contents and substring-only
matches preventing prefix completion. The bounded correction below has passed
independent review and manager acceptance at `initial-prefix-050cdf8c`; the
accepted successive package remains frozen.
NPD-C09 is accepted at implementation `35d5e014af701ea505dd0772156618cbb78d5627`
and protected visual-owner commit `93e94e0c4ce04bde7db381d8f1a994a7b15ff28f`.
The final acceptance below closes the doorway delivery at its reviewed scope;
complete qualification remains separate. No successor is selected by closure.
Historical Project/Package evidence keeps its original source. S5A remains paused.

## Approved terminal handoff correction — 2026-10-18

<!-- AUTHORITY:NATIVE-PROJECT-DOORWAY:NPD-C09-TERMINAL-HANDOFF -->

After successfully creating a Project in the f133 QA build, the owner reported
that the terminal panel collapsed and the selected Home shell said “no Project
bound”. The owner approved the proposed correction: “I approve/agree please
proceed”. This extends the current NPD-C09 assignment; it does not reopen the
accepted file-actions package or authorize a successor.

Preserve the terminal panel's visibility, chosen height and maximized/restored
state across New/Open. Successful publication selects the prepared Project-root
terminal with its own context; existing Home/AI/other-Project sessions retain
processes, cwd, environment, identity and scrollback. Closed panels stay closed
and reopen on the selected Project terminal. Keyboard focus remains with the
editor after publication. Cancellation/refusal does not publish a session.
Default labels and selected-session chrome identify Home/independent versus the
actual Project, preserve explicit rename/child titles and lifecycle feedback,
and distinguish valid current cwd from a fallback launch directory. No automatic
shell command, rebinding, restart or migration prompt is introduced.

Decision 005 and Native Terminal specification §5.2 record the authority.
Claude owns a bounded current addendum in `start-page-study.html`; the retired
terminal study remains historical. The retained umbrella lease was explicitly
renewed for the same manager/session; all prior dirty work and the f133 package
are preserved. The dev-agent owns native implementation/tests, the manager owns
state/authority reconciliation, and independent review remains required.
Acceptance requires ordinary native Wayland/WGPU New/Open with preserved panel
geometry and original live shell, clear per-session context/cwd, focused negative
and positive controls, independent rendered/source/runtime review and a new
pinned QA launcher. Existing protected landing and broader qualification limits
remain separate. This paragraph records authorization, not implementation PASS.

The manager released native translation after independent source and seven-frame
review of Claude's final target
`6780aaa270dd1666a5682da4b834cbed088e311bc68c8ac8ea87808a44429bab`.
The earlier file-action states remain byte-identical. Closed panels retain their
tab strip and selected-session context; available width is used before labels
are shortened. At 620 px, the compact close confirmation retains visible Project
and directory stubs plus the existing TERMINATE/CANCEL actions. Full context and
keyboard instructions remain accessible. The target adds no Restart control;
the existing Ctrl+Shift+R command remains. The final copy correction changes no
captured frame bytes, so the seven b405d4 captures are reused. Receipt:
`.datum/native-project-startup-repair/c09-terminal-handoff-prototype/final-copy/independent-review.json`.
The native correction and QA package are now accepted at their bounded scope.

<!-- EVIDENCE:NATIVE-PROJECT-DOORWAY:NPD-C09-TERMINAL-HANDOFF-QA -->

Launch the pinned build from an active Wayland desktop:

```sh
/home/bfadmin/Documents/datum-eda/target/qa/terminal-handoff-ec2cd61b/launch.sh
```

The source manifest is
`ec2cd61b8e2e9e5d5b123d9f3a8e910d68baedf684eca6bd77cf01500f14fbdd`
(96 Rust files and three proof helpers); the GUI SHA-256 is
`0a0bbb435d07ce2a36492bd2a1613eb99aeadfe49de50c375b7ba9af27263a46`.
The CLI and daemon reuse the unchanged f133 binaries. Source is an uncommitted
worktree snapshot based on `9857969199fc16545d843ecb516bd6ad5a15fe2f`.

Ordinary native New/Open demonstrated unchanged dock bounds before any repair
click, selection of the new Project-root shell with editor focus, and preservation
of the original live Home shell and child. Selected context distinguishes actual
Project binding from an independent shell and reported cwd from the actual
launch-directory fallback. Nine focused successful batches record 28 passing
executions across 27 distinct cases. Closed/maximized/cancel/stale publication
coverage is a focused Runtime proof, not ordinary menu qualification.

The full workflow uses the qualified ff3b GUI evidence; final production changes
are local renderer fitting only. Final-binary native evidence separately verifies
Open, complete compact close confirmation at a 620 px client, full semantic
instructions, Cancel preserving the shell, and owned PTY exits. The actual
delivered launcher passed from Home with native Wayland/WGPU Vulkan. Independent
review inspected all 18 native/launcher captures and verified the final package,
source pins, actual logs and qualified reuse. All six mandatory review findings
are resolved. Compact evidence and exact limits are referenced by the existing
JSON record; raw receipts remain ignored task scratch.

Independent disposition: `BOUNDED_SOURCE_RUNTIME_RENDERED_PACKAGE_QA_PASS`,
`.datum/native-project-startup-repair/c09-terminal-handoff-independent-final-review.json`
SHA-256 `1cc744b00773d17623e25cb0a1d07cb01aaea35f13c710afba2a642b9c01c140`.
Manager disposition: `ACCEPT_BOUNDED_TERMINAL_HANDOFF_QA_PACKAGE_LANDING_BLOCKED`,
`target/qa/terminal-handoff-ec2cd61b/manager-review.json`
SHA-256 `0cc0bc42d35ea2abd1ac5c0e581a3400629d6df1256a23835624d552d6d2cef3`.

Related inherited defects remain separate intake: View Maximize focus dispatch
(`dat-terminal-maximize-menu-focus-5iu3`), a retired session's exit notice labeling
a surviving shell (`dat-terminal-retired-exit-status-00ah`), and custom profile
launch-directory host protection (`dat-terminal-profile-launch-protection-sgh9`).
No idle lifecycle truthfulness, scrolled-viewport, live AI-agent, full AT-SPI,
ordinary window-Close or complete platform qualification is claimed. DISPLAY is
retained for the existing clipboard bridge; Datum itself uses Wayland. Broader
Clippy/convergence/source-health limits retain their existing scope.

At this terminal-handoff checkpoint, required landing/reconciliation remained.
NPD-C09 now also includes the subsequent manifest-activation correction below. The
protected visual-owner commit gate is unresolved: the authorized visual-owner
session must land its reviewed prototype through the normal gate, followed by
reviewed route reconciliation and frozen source/state landing. No gate bypass,
digest refresh to bless unauthorized output, task closure, S5A or successor
release is implied by QA acceptance.

## Open Project manifest activation correction — 2026-10-18

<!-- AUTHORITY:NATIVE-PROJECT-DOORWAY:NPD-C09-MANIFEST-ACTIVATION -->

Owner QA confirms that the terminal stays open after New/Open and reports that
double-clicking the existing `project.json` entry does not open the Project.
The shared activation path currently admits only navigable directories, leaving
that visible file inert. Continue the current NPD-C09 delivery under this owner
correction: in Open, double-click or Enter on an exact regular `project.json`
entry invokes the existing Open command with its containing directory. This is
a shortcut for the same candidate, validation, refusal and session-publication
path as the Open button. It is not proof that any matching filename is valid.

Ordinary folders keep navigation behavior; highlighting and passive browsing
never publish. New Project, arbitrary files, similarly named entries and links
do not acquire file-opening behavior. All pending host-action, native-edit,
existing Open, stale-listing and source-admission guards remain. Reset a pending
double-click when navigation or a new scan changes the listing, and never queue
a refused Open for later automatic execution. Refusal preserves the current
Project and terminal sessions. Share activation policy across pointer/keyboard
and announce it accurately through existing browser accessibility and hints.

PM063 records the bounded exception to the prior general file rule. The actual
Claude owner reconciles the protected interaction target; the dev-agent owns
native source, focused tests and ordinary native Wayland proof; the appointed
manager owns state and acceptance with independent review. Prove actual manifest
double-click and Enter, correct containing-root selection, invalid/refused Open,
pending/stale guards and ordinary-folder/nonmanifest/New preservation. Reuse
adequate unchanged evidence. Deliver a new pinned QA copy; keep the accepted ec2
source/binaries and earlier packages unchanged. No broader terminal repair,
file association, dependency, owner-data writes or protected landing bypass is
included. The manager accepted actual independent source/authority and ten-frame
review of Claude target `22e1edc62c33437ea94073f4459cb60ebf95075456956af819b9962b38ce3e59`
and released native translation. Existing CSS and New/terminal frames are
unchanged; relevant Open copy is consistent across shared examples. The retained
Open command is path-based: it checks resolved source/history during preparation
and admission, without a discovery-time inode/Project-identity promise.

<!-- EVIDENCE:NATIVE-PROJECT-DOORWAY:NPD-C09-MANIFEST-ACTIVATION-QA -->

The correction and pinned QA package are accepted at bounded scope. Launch from
an active Wayland desktop:

```sh
/home/bfadmin/Documents/datum-eda/target/qa/manifest-open-94d59290/launch.sh
```

The source manifest is
`94d5929016fa415ec3564ffa60818b568877717613c6f79bee594e011cc0c735`
(97 Rust files and four proof helpers); GUI SHA-256 is
`8a1f61a7650c41b310300a469f27e0c1ea80e1e03f56157fbf295bd094dfc913`.
The guarded build used checkpoint 3; the final checkpoint changes only the proof
helper's row lookup, with all Rust hashes unchanged. CLI/daemon reuse ec2 binaries.

Ordinary native Wayland/WGPU testing exercised manifest double-click, list Enter,
an interrupted click remaining passive, and readable malformed-manifest refusal.
The new shells use the containing Project root; the existing Home process and
shell variable survive. Dock bounds and existing sessions survive refused Open.
The executed Project file hash set is unchanged. This is Open-only evidence;
prior New evidence keeps its original scope. Focused logs record three passing
executions across two distinct cases; the final controller run supersedes the
intermediate run. Review findings for interrupted clicks and refusal lifetime
are resolved. The first native attempt's row-selection driver error is retained;
the corrected driver passed against the unchanged GUI binary.

Independent review inspected all ten final workflow captures and the actual
launcher capture, plus source, tests and package bindings. Receipt:
`.datum/native-project-startup-repair/c09-manifest-independent-final-review.json`,
SHA-256 `40daca8d2260b0aa5077a36ab0f23526c1a2c37ba18d69ddd4ee9af9f5acd5fc`.
Manager acceptance:
`target/qa/manifest-open-94d59290/manager-review.json`,
SHA-256 `fe605554300cd1b75d5e7d74eec245ac4abb496329a0813bef894aaccb73652f`.

Source remains uncommitted. The protected visual-owner landing gate, the three
unchanged HEAD-relative source-health failures and broader qualification limits
remain open. Owned nested Wayland evidence does not qualify KWin end-to-end;
final GUI SIGTERM does not qualify ordinary window Close. No task closure, S5A
or successor release follows from this QA acceptance.

## Shared Location assistance — 2026-10-18

<!-- AUTHORITY:NATIVE-PROJECT-DOORWAY:NPD-C09-PATH-ASSISTANCE -->

Owner QA confirms manifest double-click now works and reports an immediate
missing-folder error while typing `/home/bfadmin/Doc`. The owner requests adaptive
text filtering/find and Tab completion to simplify New/Open. Continue the
existing NPD-C09 assignment with one shared Location/query controller and the
existing browser worker, editor, list and accessibility projection. PM063 records
the bounded interaction; Menu and Preferences Surface §9 consume it. This is
local directory assistance, not recursive search or new engine behavior.

Partial text produces matching-folder suggestions and neutral no-match feedback.
Case-insensitive literal substring matching ranks prefixes first and retains
exact names. Explicit Tab or selection followed by Enter/double-click updates Location; arrows
only select. Ordinary Tab
traversal remains available and Shift+Tab always traverses. No automatic Project
publication or silent parent substitution occurs. Final Create/Open still uses
the exact typed candidate and existing validation; actual explicit failures stay
truthful. Stale replies, click pairs and selections cannot overwrite later input.
Only an exact ready listing permits host naming/actions or manifest activation.
Preserve the current draft, automatic Units snapshot, host receipts and sessions.

The same manager lease is renewed; dev-agent owns implementation/proof, actual
Claude owns the bounded visual target, and closeout_review remains independent.
Acceptance requires actual shared New and Open typing/completion, ambiguous and
no-match handling, keyboard escape/traversal, explicit invalid-path refusal,
stale/guard regressions and ordinary native Wayland/wgpu workflow evidence, plus
a new pinned QA launcher and independent rendered/source/runtime/package review.
Reuse unchanged accepted 94d behavior without relabeling prior evidence. No
owner data, new dependencies, additional search service, broad terminal changes,
protected landing bypass, task closure, S5A or successor authorization is included.
The manager accepted actual independent authority/visual review of Claude target
`85b2265862e6f8eb271c3508e01a55ac555855aa6f7aec2fc61aea5717295c31` and released bounded native implementation.
Ten actual captures at4500 retain identical frame markup/CSS in final85b2; the
final changes reconcile explanatory keyboard and refusal copy only. Receipt:
`.datum/native-project-startup-repair/c09-path-assistance-prototype/independent-review.json`,
SHA-256 `f2edec1d433f6d1824ab2444d83f6530e2ea89aa5608a8ca4bf48ec99cade0f1`.
Native implementation, runtime and the new QA package are accepted at the
bounded scope below; source landing and full qualification remain open.

<!-- EVIDENCE:NATIVE-PROJECT-DOORWAY:NPD-C09-PATH-ASSISTANCE-QA -->

Launch the reviewed build from an active Wayland desktop:

```sh
/home/bfadmin/Documents/datum-eda/target/qa/path-assistance-e12d125d/launch.sh
```

The final source manifest is
`e12d125d1112edab4e470bc0cb8b7835d9f232be5daa10225053d4f50d82b2b8`
(104 Rust files and five proof helpers); the GUI SHA-256 is
`8f9aec9a34208ceeaad295d800ae89c1cfd15ef2d90cc65f16c58cb43fee6005`.
All Rust sources match compiled checkpoint 5. Checkpoint 6 corrects only the
proof helper's owned-dialog resize; no rebuild is inferred. CLI and daemon
reuse the unchanged accepted 94d binaries.

Four distinct focused cases pass: two model, one scanner and one shared Runtime
case. The Runtime verifies actual selection/acceptance announcements and
truthful refusal when a suggested folder disappears or is replaced by a file.
Ordinary native Wayland/wgpu New/Open demonstrates partial matching, ambiguous
Tab traversal, selection/Escape, exact completion, neutral no-match input,
explicit invalid-path refusals, actual Create/Open and preserved terminal
sessions. The fresh Project's five-file hash set is unchanged through Open.
Independent review inspected all 15 workflow captures and the actual delivered
launcher capture. The New dialog's long-query capture has a verified 620×620
client, wrapped status and reachable footer; this is a bounded layout check.

Independent receipt:
`.datum/native-project-startup-repair/c09-path-assistance-independent-final-review.json`,
SHA-256 `10dac659904008e1e20540ab36e2ef70939499d666774588a836fa399a317036`,
disposition `BOUNDED_SOURCE_RUNTIME_RENDERED_PACKAGE_QA_PASS`.
Manager receipt: `target/qa/path-assistance-e12d125d/manager-review.json`,
SHA-256 `f1b8e8d3dabd2bd64b8e76b56033844f57db377fea8a1a3709ae4e732417284a`,
disposition `ACCEPT_BOUNDED_PATH_ASSISTANCE_QA_PACKAGE_LANDING_BLOCKED`.
The existing paired JSON binds source, build, tests, native results and the
implementer handoff. Earlier compile, test-focus and harness-resize failures
remain recorded; the cropped first attempt is excluded from minimum evidence.

Exact Unicode/space path completion passes, while the known missing lambda
glyph remains `dat-ui-filename-glyph-coverage-o53p`. This is neither full font
coverage nor full accessibility/platform qualification. The owned nested
Wayland/Vulkan workflow does not establish KWin end-to-end or nonempty history
qualification; final GUI SIGTERM after owned shell exits is not ordinary window
Close. Protected visual-owner landing and the same three inherited HEAD
source-health failures remain unresolved. NPD-C09 stays in progress; no gate
bypass, waiver, task closure, S5A or successor release is implied.

### Successive Location completion correction — 2026-10-18

<!-- AUTHORITY:NATIVE-PROJECT-DOORWAY:NPD-C09-SUCCESSIVE-COMPLETION -->

Owner QA confirms initial Documents completion but reports that the next
`datum-ed` search/completion fails. The screenshot shows exact `datum-eda`
without a trailing separator and its children. Two coupled rules explain this:
Tab refuses a common-prefix extension that itself names a folder, while typing
that exact name immediately ends the query and replaces siblings with children.

The manager continues the existing NPD-C09 grant. While editing an unslashed
final component, retain the parent query even if that exact folder exists.
Common-prefix extension may reach that existing name without entering it or
choosing among siblings. Explicit selected/unique acceptance, Enter/navigation,
or a typed trailing separator enters the directory. For an explicitly loaded
unslashed directory, eligible Tab appends only a separator to the same path.
Roots and supported home/dot anchors keep their resolver meaning. Preserve exact
Create/Open candidates, host/manifest guards, stale-result rejection, drafts,
Units and current Project/terminal sessions.

PM063 owns this clarification; Menu and Preferences Surface consume it. Actual
Claude reconciles explanatory copy while existing query/accepted layouts are
reused. Dev-agent implements and proves successive components in both New/Open;
closeout_review independently checks affected source and native evidence. The
ordinary proof must traverse Documents, ambiguous `datum-ed`, exact-name sibling
matches and a further child before canonical Create/Open. Reuse unaffected e12d
tests and visual evidence without relabeling their scope. Owner processes/data
remain untouched. No new dependency, engine semantics, protected landing bypass,
waiver, task closure, S5A or successor release is authorized.

<!-- EVIDENCE:NATIVE-PROJECT-DOORWAY:NPD-C09-SUCCESSIVE-COMPLETION-QA -->

The correction is accepted for bounded QA. Launch the current reviewed build
from an active Wayland desktop:

```sh
/home/bfadmin/Documents/datum-eda/target/qa/successive-dee25e1e/launch.sh
```

Its source manifest is `dee25e1eb3d6eeef0dd78391234aa93d8ea6aee980512ca0d30c0d796bb04b21`
(104 Rust files and five helpers); GUI SHA-256 is
`cf7587434c4a65110610cc4ea24fa043e5b31a578c6a510bfb55d93673ef53fd`. Seven Rust owners and
one existing helper differ from e12d; all 101 other pinned sources are unchanged.
Five focused cases pass: three models, the scanner and the shared New/Open
Runtime. The ordinary native workflow completes Documents, appends `datum-ed`
at the existing caret, retains all three siblings after Tab reaches `datum-eda`,
explicitly accepts a folder, then appends and completes its next component in
both dialogs. Twelve workflow captures and the actual delivered launcher frame
were independently inspected. Canonical Create/Open preserves the five fresh
Project hashes and original shell PID/variable; all three owned PTYs exit.

Independent receipt: `.datum/native-project-startup-repair/c09-successive-independent-final-review.json`,
SHA-256 `c1d1f79db7fe74bc0b8756240e3780e0c8cb4468c9d5323e65d910dcdcdc56ce`. Manager receipt:
`target/qa/successive-dee25e1e/manager-review.json`,
SHA-256 `f6744f7a8c97bd4d13092c585ea77e3fcecf4ab54d3603215795966ff64f3709`. Both accept bounded QA only, with no
mandatory findings. The paired JSON binds the actual build, native commands,
results and qualified reuse. Claude target `b7193cd7` changes explanatory copy;
existing frame/CSS evidence retains its prior scope. No fresh minimum-size,
Unicode glyph, nonempty history, KWin desktop or ordinary window-Close claim is
added. Protected visual-owner landing and the three inherited HEAD source-health
failures remain unresolved; NPD-C09 remains in progress and S5A stays held.

### Initial listing and prefix-priority correction — 2026-10-18

<!-- AUTHORITY:NATIVE-PROJECT-DOORWAY:NPD-C09-INITIAL-PREFIX -->

Owner QA reports that Open initially shows Home without its contents and that
`~/Documents/dat` followed by Tab moves focus to Browse despite four `datum*`
prefix matches. Initial navigation is collapsed and never starts a scan. The
completion model includes substring-only `plugdata` when calculating a shared
prefix. Continue the same NPD-C09 grant to correct both shared New/Open owners.

Show the existing navigation immediately and issue a strict asynchronous read
of the resolved starting directory. Keep initial field focus and distinguish
loading, empty and unreadable states. Selected suggestion completion retains
priority; otherwise compute completion from the prefix group when nonempty,
keeping substring-only matches visible for explicit selection. With the owner's
four prefix names, the exact common prefix is `datum`, because `datum_projects`
does not share the hyphen. Preserve the no-prefix fallback, exact filename
spelling, candidate isolation, stale-result and host/manifest guards, drafts,
Units and Project/terminal sessions. Browse stays visible and focuses the list.

PM063 owns the correction; Menu and Preferences Surface consume it. Actual
Claude owns the existing prototype's initial expanded states and explanatory
copy reconciliation. The nonvisual completion model and its focused regressions
are released under the amended PM063 now; initial visible-state translation
follows target review. Closeout_review independently reviews the bounded
source/runtime/rendered package. Verify both
dialogs before any Browse click or text edit, then ordinary successive typing,
prefix-group completion, canonical Create/Open and session preservation on
native Wayland/wgpu. Reuse unaffected accepted proof; use only owned scratch.
No new dependency, engine semantic change, protected landing bypass, task closure,
S5A or successor release is granted. Previous accepted packages remain frozen.

Actual Claude target `cf6dc2b09a89f3cbdc1282f330ef533c26262aaf4d1c94b2c1eae34cfa4beef7`
is cleared for strict Runtime translation by closeout_review's actual independent
bounded target/authority PASS on 2026-10-18. All 21 PNGs across 11 changed states
were inspected, including New/Open default/minimum and refusal/footer/scroll.
The successful attempt-2 receipt and actual Claude copy receipt are bound in the
companion JSON. The first capture failed in the read-only helper on an existing
wrapper and remains excluded; the corrected helper completed. All 41 frame spans
and three style blocks match the captured source; final explanatory copy alone
clarifies that `plugdata` drops out after the `datum` query refresh. The example
Project name supplies no runtime default-name authority. The manager releases
initial-directory translation, focused admission regressions, serial guarded
build and ordinary native New/Open plus actual-launcher proof. Prefix-model
regressions passed four focused cases at that release; final acceptance follows.

The implemented correction has actual independent bounded source/runtime/rendered/
package PASS and appointed-manager QA acceptance at `initial-prefix-050cdf8c`.
The companion JSON binds both receipts and the final source/build/launcher.
Six distinct focused cases pass. Fourteen ordinary native Wayland/wgpu/Vulkan
frames show initial New/Open Home contents before Location edits or Browse,
correct initial field focus, `dat` five rows followed by Tab to `datum` four rows
with Location focus retained, further folder completion, canonical Create/Open,
and preservation of the original terminal. The actual delivered launcher also
passes with its packaged executable and Home cwd. The manager verified 36 handoff
bindings, 109 current/archive source pins and seven representative native/launcher
images. The reviewer inspected all fourteen native images and the launcher image.

Real initial worker replies are rejected after edit/cancel/reopen, and created-root
replacement admits the intended owned golden Project while preserving its four
canonical files. The native Create/Open preservation check covers four fresh
Project files plus an empty writer lock; it supplies no nonempty journal/history
qualification. Initial Home names/types are authorized read-only discovery; all
host mutations and Project proof use owned scratch. Default-size native proof,
reviewed minimum prototype captures and qualified unchanged minimum evidence
retain their distinct scopes. No mandatory finding remains for these corrections.
At this QA checkpoint, source was uncommitted and protected visual-owner landing
and three inherited HEAD-health findings were unresolved. The later landing
record below supplies their current disposition; this checkpoint grants no
source-landing PASS, task closure, full qualification, S5A or successor release.

Owner QA follow-up on 2026-10-18, after delivery of `initial-prefix-050cdf8c`:
“QA testing finds that the issues have been resolved and the open/new project is
making more sense now.” This records hands-on confirmation of the reported
initial-directory display and Tab-completion corrections and clearer New/Open
usability. Existing package receipts, independent review and separately open
landing/qualification retain their recorded scopes.

## Owner-authorized landing — 2026-10-18

<!-- CHECKPOINT:NATIVE-PROJECT-DOORWAY:NPD-C09-LANDING -->

After confirming the QA corrections, the owner explicitly authorized committing
the dirty-tree changes and pushing the project, then consulting the roadmap for
its next development step. The paired JSON preserves the exact owner statement.
This authorizes normal landing; the protected visual-owner gate remains applicable.

The three source-health findings from the QA checkpoint are corrected against
original base `98579691`. Shared shell and recursive pane geometry now live in
a real normal layout module with preserved root exports. Five unchanged renderer
test modules follow their implementation owners. Renderer root physical lines
fall from 129 to 128 and include-expanded lines from 3,177 to 2,657. The existing
editor-command extraction reduces protocol production lines from 2,887 to 2,842;
its 1,705 inline-test lines remain unchanged. Both ledger ceilings ratchet exactly
downward; no policy, budget, exclusion or comparison-base change is introduced.

Source health passes for 3,248 files and all 13 policy regression tests pass.
A current terminal boundary check found an obsolete parent-file reference for
accessibility refresh. The corrected checker requires the actual parent wiring
and child refresh/core-snapshot route; all 16 hermetic tests and the current
boundary pass. Dependency authority passes. Actual independent source review
finds no mandatory issue in the frozen extraction or checker correction.
The first renderer compile exposed an internal cache-owner visibility error;
the sole correction makes shell_layout visible within the crate and preserves
its existing root path. The corrected batch compiles: 34 tests pass and one
inherited assertion fails because it treats a legitimate sidebar ReviewAction
as an editor-scene hit. Independent source analysis confirms the relevant test,
sidebar painter and geometry solvers are unchanged from original `98579691`.
This is intake `dat-terminal-hit-sidebar-assertion-agz0`; later assertions in
that failed case were not reached. No original-base runtime replay is claimed.

The matching GUI build and ordinary native Wayland/wgpu/Vulkan New/Open workflow
pass. Fourteen captures cover initial contents, successive prefix completion,
canonical Create/Open and preservation of the original terminal. The manager
verified all 27 handoff artifact hashes and four representative captures.
The single full drift attempt exits 101 at the existing 16 Clippy errors in
eight unchanged engine files (`dat-lk0l`); later full-battery gates were not
reached. Workspace formatting and resource-policy checks before Clippy pass.
The failed compile and test attempts remain retained. Full qualification stays
open. Actual independent final disposition is
**BOUNDED_SOURCE_NATIVE_RENDERED_LANDING_REVIEW_PASS**, with no mandatory finding.
The receipt is `.datum/native-project-startup-repair/c09-landing-health-independent-final-review.json`
(SHA-256 `097810226e0e3847458adef3911769fa5d74f099bb6b1a34ad95f91237b092e5`).
The manager accepts this bounded review. All 121 reviewed source/checker files
landed locally in `35d5e014af701ea505dd0772156618cbb78d5627` through normal
file-lane, temporary-artifact and staged-formatting hooks. Every committed blob
matches its reviewed pin; the index was empty after that commit. At this
intermediate implementation commit, visual/route/task closure remained pending.

The first actual visual-owner session verified the reviewed prototype, but automatic
approval refused Git inspection because approval was required without an available
approval interface. It did not stage, commit or modify the prototype. The owner
then used the existing authorized visual-owner session to land only that exact
file through normal hooks. Commit `93e94e0c4ce04bde7db381d8f1a994a7b15ff28f`
contains only `docs/gui/prototypes/start-page-study.html`, directly atop
`35d5e014`. Its committed and live SHA-256 is the accepted
`cf6dc2b09a89f3cbdc1282f330ef533c26262aaf4d1c94b2c1eae34cfa4beef7`.
The manager and independent reviewer verified the commit, parent, sole-path
scope and exact bytes. The earlier refusal remains recorded in the paired JSON;
the authorized owning-lane commit resolves it without a gate bypass.

### Final C09 objective acceptance

<!-- EVIDENCE:NATIVE-PROJECT-DOORWAY:NPD-C09-LANDING-ACCEPTANCE -->

Appointed manager `codex-reorganization-manager-20261014` accepts C09 under the
existing owner direction and PM059. The accepted implementation and visual
commits above deliver the approved shared New/Open browser, automatic Units
inheritance, protected New Folder/Rename actions, terminal panel/session handoff,
manifest activation, initial folder contents and successive path completion.
This disposition relies on the actual independent source/native/rendered/package
reviews recorded here, the final landing review `09781022…`, and the owner's
hands-on QA confirmation. Frozen packages and their original source bindings
remain unchanged.
The paired JSON maps each current workflow and safety requirement to its accepted
cohort, distinguishing the final native regression from qualified earlier reuse.
Its complete earlier C08 record is preserved under `historical_c08_acceptance`.

The independent reviewer verified complete member coverage for all six affected
evidence routes. Unchanged members reuse their exact accepted pins; later changes
to the shared-browser decision, Preferences surface, menu bindings, prototype and
these delivery records have actual bounded review. Their digests are reconciled
only after the authorized visual commit. Route classifications retain their
existing scope. The source-health parity row records the two reviewed downward
ceilings without changing policy.

C09 and `dat-native-project-startup-vrf` close with implementation `35d5e014`
and visual `93e94e0c` as landing evidence. The Frontier claim is released and
bounded development availability is recorded. Existing engine Clippy
(`dat-lk0l`), renderer assertion (`dat-terminal-hit-sidebar-assertion-agz0`),
broader host/assistive/history/close qualification and S5A hold remain unchanged.
The owner authorized normal commit and push; this local acceptance does not
claim that remote publication has already occurred.

<!-- DISPOSITION:NATIVE-PROJECT-DOORWAY:C09-CLOSED -->

The completed doorway has no authorized automatic successor. Set explicit idle
after closure; a fresh roadmap query reports that state. S5A remains held, and
intake defects remain intake until an explicit Frontier disposition selects work.

## Actual owner clarification and manager scope

The owner clarified to the appointed manager: “this purposial requires inpu
from me the owner. I wanted to remove myself from this development process.”
This follows the actual appointment recorded in
[initial reorganization](../specification-history/native-delivery-reorganization.md#appointment-and-bounded-authority).
Manager `codex-reorganization-manager-20261014`, thread
`01a13df1-7a0d-72c1-bbdd-f0d96d481598`, accepts responsibility for ordinary
startup presentation/post-create decisions, scoped NPD-C02 coordination and
objective NPD-C06 acceptance under this exact clarification and PM059. This
supersedes routine owner handoff in the earlier recommendation; it is not a
claim that the owner supplied an implementation or HUMAN PASS.

The initial preparation assignment selected existing NPD-C01 and scoped planning lease, not a new
administrative task. Prepare current-code/application observations, settled
startup/Preferences/visual authority, complete bounded behavior/fixture/refusal
contract and exact NPD-C02 packet for the manager. At that preparation stage implementation remained
unauthorized until the exact manager disposition now recorded below. Full NPD-C01..06,
independent review and actual native proof remain; no owner-relay approval loop.

S5A remains paused, unclaimed, with retained grant/evidence/unfinished steps;
only its selection flag changes for this assignment. No source rights,
third-party dependency, unrelated product rule, waiver, destructive change,
protected Claude prototype edit or remote publication is authorized. Genuine
HUMAN evidence is recorded only if actually obtained. Existing settled designs
must be reused before proposing a new choice; any genuinely needed prototype
change goes through the real Claude lane, never impersonation.

## Historical baseline observation and reuse

One bounded existing-build no-argument observation ran the ordinary
`target/debug/datum-gui` executable under X11 with task-owned external
XDG config/data/cache and diagnostic output. It returned exit1 in 2.43 seconds:
`launch state load failed: resolve GUI launch review context: --project-root,
--board, --schematic, or --demo-known-good is required`. Diagnostic order ended
at request resolution, before window/runtime creation. No Project was created.
Binary SHA-256:
`d7708695569a53ac186cf34e4c6512f5c75d4bcde8badc36b1fea69d0ef34b42`.
Existing local debug build was reused; this is not a freshly recompiled
source/binary-cohort qualification. Current bootstrap/event-dispatch source at
`1f9bef8d` independently shows the same path. Raw observation is task-owned
external scratch, not tracked proof/log storage or a permanent availability
promise. This failure establishes the missing connection, not a doorway PASS.

Retain existing owners:

- `application/app_bootstrap.rs` and `app_native_events.rs` load a project before
  constructing Runtime/window; no-project application startup is missing.
- `windows/global_preferences_runtime/new_project.rs` already previews the typed
  Units source and calls engine genesis; success currently reports creation and
  closes its form. Do not recreate the form, preference resolver or genesis.
- `windows/project_open/{prepare,publication}.rs` and protocol `project_open.rs`
  already prepare/revalidate/admit/publish a native Project, but current-origin
  capture and publication require an existing backing root. Extend this shared
  path to a real no-current-Project application session; never use a fake root.
- Existing GUI edit client, journal/Save and accepted bounded board/schematic/
  library deliveries remain reusable at their exact source-qualified scope.
  The S3 New-form receipt demonstrated native presentation/input but explicitly
  created no Project; do not count it as create/open/reopen proof.

## Settled behavior and visual authority

The adopted foundation contract's doorway/F6/F8 requires no-argument native
New/Open/Cancel, collision refusal, immutable eligible genesis seed receipt,
source/journal reopen and prior-state preservation on refusal. PM039 disables
Project Preferences without a Project. Global Preferences product surface §9
and PM040 own the existing normalized genesis request: Global Units default,
explicit Factory alternative, eight current Units only, no implicit fallback,
no activation of reserved templates/startup/recent/other seed descriptors.
Create alone publishes; genesis is not an undoable design edit.

`start-page-study.html` lines111/146 explicitly defer the full Start page;
lines147–156 independently govern the existing New Units form. Do not activate
the reserved startup-view descriptor, recents, Last session behavior, templates,
welcome panel or a startup network operation. The File menu's existing New/Open
commands and current chrome/empty/disabled states are the reuse boundary.

Actual owning-lane read-only consultation through the installed CLI returned in
27.6 seconds (session `b86a2b55-47c2-4c00-a94a-20403ba8e725`). It used supplied
exact source excerpts with tools disabled, not independent file reinspection,
full-route review, approval or HUMAN/runtime PASS. The earlier broader request
timed out after150 seconds without response; no retry loop or source edit.
Advice recommends empty Main/existing File New/Open, create→existing Open worker,
and preservation of a successfully created Project if subsequent Open fails.
No protected HTML change is needed if only existing chrome/empty/disabled states
are reused and no new fixed startup/welcome text, CTA, recents or visual widget
is introduced. Truthful dynamic names, paths, state and refusal explanations
inside existing governed controls remain required. An
actually uncovered visual element requires a bounded real Claude-owned change,
not a native improvisation or marker bypass. No HTML was modified.

## Historical pre-disposition NPD-C02 recommendation

Choose an empty, operable Main application state on no-argument launch, with
existing File→New Project/Open enabled and project-scoped actions disabled through
shared admission. It has no design, fabricated Project/model/UUID/scene truth,
persisted placeholder, automatic sample, recents, or adopted preference. Reuse
existing chrome; no new fixed startup/welcome copy, widget or centered action surface. Preserve existing
explicit project/import/demo launch modes and their own contracts; no fallback
from an explicit invalid source to an invented Project. The ordinary GUI Open
path remains correctable without exiting or replacing a valid Project.

After successful engine Create, use the actual typed result/root to prepare and
publish through the one existing Open worker. Do not parse console text, create
another loader or privately write source. If Open then fails, keep the genuinely
created Project and prior application state, report the two outcomes truthfully,
and permit an explicit retry/Open. Report both outcomes through existing
`ConsoleFeedback`/Console and `ui.project_open.refusal`/`ui.new_project.refusal`
slots, owned by the New/Open controllers, existing form scene and shared Console
presentation. Dynamic Project names/paths and clear created-but-unopened/error
explanations in those slots are required content, not a new dialog or welcome
layout. “No new text” excludes new fixed startup/welcome content, never truthful
existing-channel feedback. No destructive rollback/deletion or repeated genesis. Before Create, Cancel/refusal leaves source/journal unchanged. After
published genesis, cancellation cannot pretend that creation never occurred.

No-current-Project origin qualification is transient session state: preserve
request/session-generation and stale-result refusal without a fake filesystem
origin. Existing-project transitions keep root/revision/writer protections.
Prepare resources off the input thread; revalidate exact source/history before
publication and reject cancelled/superseded results. Do not publish half a scene,
terminal or Project context. Reuse existing terminal/session owners; no Project
variables/capability or synthetic Project root before a Project exists. Existing
terminal semantics, provider/dependency choices and S5A are outside this change.

### Production consumers and source health

Proposed implementation scope, only after C02:

| Owner | Bounded work |
| --- | --- |
| gui-app application bootstrap/events/runtime | Explicit no-project application state and existing-window launch; normal project launch preserved. |
| gui-protocol workspace state/loading/project_open and shared menu admission | Truthful absent Project, guarded no-project origin, empty/disabled consumer projection; no model/persistence schema or private writer. |
| gui-app New/Open controllers/publication | Typed successful Create result into existing Open worker; cancellation/stale/prior-state preservation and truthful partial outcome. |
| Existing terminal/session context owner | No fabricated Project context while none exists; preserve established project switch/admission/cleanup. No parser/PTY redesign or external library. |
| Existing gui-render shell/frame owner, only if needed | Existing chrome/empty/disabled presentation without new visual elements; request actual owning-lane reconciliation for an uncovered state. |

All source-health limits remain. Protocol lib.rs has legacy production/inline
ceilings2940/1705; do not grow it. If its registration/construction changes,
extract the whole existing ReviewWorkspaceState constructor into its actual
workspace-state owner and ratchet the measured legacy ceiling downward in the
same change. No forwarding/include split or ceiling exception. New cohesive
source/test modules stay within normal budgets; selected formatting and all
compilation/proof use the existing guarded serial Cargo route.

### Reproducible native fixtures and proof matrix

Use a task-owned external workspace `native-project-doorway`, isolated Global
Preferences, and ordinary GUI creation of A/B at absent destinations. Record
engine-generated actual IDs, source/profile/Units receipt, source/journal/cursor
hashes and candidate/binary/environment. No third-party library is required for
empty native creation/open. Global-default A and explicitly chosen Factory B
exercise F1/F8 copy-once isolation; no implicit Factory fallback. Negative copies
are owned derivatives only; never modify owner originals. Capture fixture values
through existing typed/query services; do not use CLI provisioning as the proof
of the ordinary GUI create/open path.

| Scenario | Required actual observation / refusal / preservation |
| --- | --- |
| NPD-T01 no-argument entry | Main stays operable with no Project/source creation; existing File New/Open reachable; Project Preferences/edit/check/output actions reflect absence honestly. No fake sample or reserved Start page. |
| NPD-T02 New/Cancel | Existing form, default Global/explicit Factory, eight-value/source summary and keyboard focus; cancel/Escape returns prior valid state with no genesis or journal change. |
| NPD-T03 Create→Open/reopen | Normalize name/location; absent-destination engine genesis/immutable receipt, then single guarded Open publication. Save/close/reopen resolves same actual identity/source/history; no imported path/terminal command required. |
| NPD-T04 seed refusals | Stale Global generation, unreadable/migration-required source and unresolved values preserve entered data/focus and refuse Create. Offer Factory explicitly, never select it; destination collision never overwrites. Reuse adequate existing engine/Preferences proof at unchanged seams. |
| NPD-T05 Open/Cancel | From empty Main and a valid A, open B through existing worker. Cancel/Escape/window close or superseded request cannot publish stale B, replace A, or leak a pending terminal. Keep attempted path/refusal for correction. |
| NPD-T06 invalid/corrupt/unsupported/busy | Missing directory/required shard, corrupt JSON, unsupported profile and held writer refusal leave valid prior Project/session untouched. No guessed migration, new source, overwrite, owner-file repair or fabricated available state. Operational coordination files remain distinguished from authored/journal state. |
| NPD-T07 recovery/partial outcome | Use existing canonical eligible committed recovery only; corrupt/ambiguous integrity remains refused. Record recovered committed materialization separately from a new edit. A successful genesis followed by failed Open remains created and correctably unopened; no destructive rollback or blind repeat. |
| NPD-T08 focus/keyboard/accessibility | Actual native focus/Tab/Shift-Tab/Enter/Escape and active-window/close/release behavior; existing semantic names/roles/refusal explanations/non-color states. Inspect available native accessibility surface and report actual limitations; do not label software events physical/HUMAN or model-only metadata a successful assistive trial. |
| NPD-T09 Project isolation | Later Global changes do not rewrite A/B source/Working Units; reopening reads current Project authority, not a UI snapshot. Read/inspect/cancel does not become a design transaction. Existing authoring and original sources retain bounded regression proof. |

Proof is current candidate-bound ordinary application behavior, not static HTML,
a parser test or clean unchanged reopen alone. Native positive/refusal tests and
distinct replay cover the approved matrix. Keep attempts/limitations, one set of
external captures/logs, and compact findings only. No speculative timing target
or full-workspace/benchmark campaign is added. Capture and inspect actual changed
visual states at appropriate ordinary/narrow/focus/refusal conditions; never
bless prototypes/goldens to absorb a mismatch.

## Required evidence versus full qualification

NPD-C04 and foundation Proof section require actual native input, visible/focus/
accessibility states, typed outcome/identity/source/history and reopen correlation;
NPD-C05 requires distinct replay. Those observations remain real requirements.
Neither source specifies a blanket HUMAN sign-off as a prerequisite to every
usable development build. PM059's Owner-directed usable development builds section
explicitly makes owner use voluntary and separates development availability from
missing platform/visual/accessibility/endurance/full qualification. PM059 never
converts a missing observation into PASS or deletes its associated requirement.

The earlier planning metadata's generic “required HUMAN acceptance before NPD
delivery” was not traced to such a requirement and is removed. Objective delivery
and routine startup decisions route to the actual appointed manager. GUI
conformance §0–1 HUMAN rows concern their named cross-engine/aesthetic reference
and golden claim; they remain unclaimed where absent and are not reclassified
by automation. Identify the exact applicable row/consumer before using one as
a blocker. No fake HUMAN/physical/assistive result, owner reply or waiver.

A proposed development availability assessment must name the actual ordinary
workflow/safety results and remaining scope. Full NPD-C04..06 closure still needs
the entire approved doorway's required proof, distinct replay and actual manager
objective disposition. Missing native accessibility or other required cases are
reported honestly against their claim; they cannot silently be ticked complete.

## Historical NPD-C02 packet for the appointed manager

Recommendation: approve the empty-existing-Main/File-New-Open integration and
successful Create→existing Open worker behavior above, preserving created data
on subsequent Open failure. Approve only this bounded runtime/source scope after
actual C01 review; no deferred Start page, source policy, dependency, budget,
protected HTML, unrelated feature or S5A grant. If a new visual element is
actually needed, route its exact reconciliation to the real Claude lane.

Use the existing exact NPD-C02 approve/revise/defer response format, received
and durably recorded by `codex-reorganization-manager-20261014` under the actual
owner clarification/PM059. No routine owner confirmation or relayed script is
required. Record the exact reviewed candidate, permitted source paths and proof
scope before changing authorization/claim to implementation. This preparation
is not that actual disposition and does not claim NPD delivery or acceptance.

<!-- EVIDENCE:NATIVE-PROJECT-DOORWAY:NPD-C01 -->
C01 preparation covers actual existing-build failure/current source, settled
versus deferred visual behavior, actual scoped read-only owning-lane advice,
complete bounded behavior/consumer/fixture/refusal/proof and C02 recommendation.
At this historical preparation checkpoint, NPD-C01 remained in progress pending
actual independent preparation review and manager disposition; this marker does not claim implementation or acceptance.
The earlier routine owner routing/HUMAN linkage is explicitly corrected in the
current Frontier, tracker design/acceptance and NPD requirement owner while prior
reorganization/WDQ receipts remain historical.

## Actual C01 preparation review and routine reconciliation

Actual distinct reviewer `/root/reorganization_review` returned bounded NPD-C01
preparation **PASS**, received by appointed manager
`codex-reorganization-manager-20261014`; no blocking findings remain.
Base `1f9bef8d06c8a266a6b4afd126b28e044a955194`;
reviewed packet SHA-256
`ac8efbe733c651948366fcbca1806262e2a1159328efeb9641f87c28811ad1df`;
patch SHA-256
`f72c6c486a7196d0d8201c0a6c094ceb368f8fdba340b27acd0dc805e8753662`.
Eight changed-file and seven runtime-inspection pins matched actual bytes.
R1-routing corrected current tracker design/acceptance to the same six IDs and
bounded manager dispositions as the spec/Frontier; original history retained.

Review verified bounded owner clarification, preserved paused S5A/other grants,
complete nine-scenario scope and29 proposed source paths, source-health limits,
actual binary/consultation limitations and created-but-unopened preservation.
It reused verified adequate complete-route review, checking25 unchanged rollout
members, one changed consumer and one added contract; unchanged foundation4 and
workspace/Preferences80 members retain their existing reviewed identities/scope.
Both complete affected consumers were inspected. No fresh whole80-member semantic
review, native/visual/HUMAN PASS or product acceptance was asserted.

The manager received the actual receipt and authorized routine recording, sole
reviewed-route reconciliation and affected final gates. The reviewed candidate
route digest was
`ec2adc7ea093eb509af150409e3102957c3f8e6703d44db68c963e76fcb90abc`.
Only this factual receipt/limits section was then appended; no behavior, scope,
requirement, decision or proof outcome changed. Recalculate the final digest
from these exact current consumer bytes, recording this metadata-only delta in
the route note. Other28 route records remain unchanged.

The narrow future-build /tmp reserve deficit was resolved by retaining owned
completed/frozen transport on disk with all original references/content hashes
preserved; guard check-only passes. No policy threshold, foreign cleanup, source
edit or Cargo build. Substantial future proof stays in the owned disk-backed
external workspace. This is not a new C01 approval condition or runtime proof.
At this historical preparation checkpoint, actual scoped manager C02 disposition
and synchronized authorization/claim were still required before source edits;
C04/C05/native evidence and all genuine
qualification requirements remain.


## Actual delegated C02 disposition and execution

<!-- EVIDENCE:NATIVE-PROJECT-DOORWAY:NPD-C02 -->

NATIVE-PROJECT-DOORWAY: approve DOORWAY-IMPLEMENTATION — NPD-MANAGED-01; independently reviewed C01 packet ac8efbe733c651948366fcbca1806262e2a1159328efeb9641f87c28811ad1df and final reconciled preparation; empty existing Main with shared New/Open, truthful absent Project, successful Create through the existing Open worker, preserved data and prior state on refusal or subsequent Open failure.

Actual disposition by `codex-reorganization-manager-20261014`, assignment `NPD-MANAGED-01`; C01 independently reviewed packet `ac8efbe733c651948366fcbca1806262e2a1159328efeb9641f87c28811ad1df`, final reviewed rollout digest `6730e4cc4464ec1b19bc40dc5c4bde570b432bc08efafe17542e810ece412180`. This is delegated manager input, not a HUMAN receipt.

Approved behavior:

- Launch without project/import/demo arguments into an operable existing Main shell with a genuine absent-Project state; File New/Open remain available and project-only commands are truthfully disabled through shared admission.
- No fabricated Project, filesystem root, persisted placeholder, sample scene authority, automatic genesis, startup descriptor activation or preference adoption.
- Preserve explicit project/import/demo launch behavior and source validation; do not silently substitute an empty or invented Project for an explicit invalid source.
- Reuse the existing New form, explicit typed Units source and engine genesis. Hand the typed successful creation result to the single existing Open worker.
- Preserve a successfully created Project and the prior application state if subsequent Open fails; provide truthful dynamic status/refusal feedback through existing governed controls and allow explicit correction/retry.
- Preserve cancellation, stale/superseded request rejection, exact source/history revalidation, writer admission, eligible canonical recovery, session/terminal cleanup and prior-state integrity.
- Reuse existing chrome/empty/disabled visual states. Ordinary dynamic names, paths and truthful outcome/refusal text are permitted in existing owned slots; no new fixed welcome surface, widgets, CTAs, recents, templates or Start page.
- Deliver and verify the entire approved NPD-T01 through NPD-T09 matrix, reusing adequate unchanged proof at its actual scope.

Permitted runtime paths (edit only necessary consumers):

- `crates/gui-app/src/main.rs`
- `crates/gui-app/src/application/app_bootstrap.rs`
- `crates/gui-app/src/application/app_native_events.rs`
- `crates/gui-app/src/application/runtime_init.rs`
- `crates/gui-app/src/application/runtime_state.rs`
- `crates/gui-app/src/application/app_shell.rs`
- `crates/gui-app/src/application/native_project_startup.rs`
- `crates/gui-app/src/application/native_project_startup_tests.rs`
- `crates/gui-app/src/windows/project_open.rs`
- `crates/gui-app/src/windows/project_open/prepare.rs`
- `crates/gui-app/src/windows/project_open/publication.rs`
- `crates/gui-app/src/windows/global_preferences_runtime/new_project.rs`
- `crates/gui-app/src/terminal/terminal_session.rs`
- `crates/gui-app/src/terminal/terminal_session_context.rs`
- `crates/gui-app/src/terminal/terminal_session/project_open.rs`
- `crates/gui-protocol/src/lib.rs`
- `crates/gui-protocol/src/workspace_state.rs`
- `crates/gui-protocol/src/workspace_loading.rs`
- `crates/gui-protocol/src/workspace_startup.rs`
- `crates/gui-protocol/src/workspace_startup_tests.rs`
- `crates/gui-protocol/src/project_open.rs`
- `crates/gui-protocol/src/project_open/tests.rs`
- `crates/gui-protocol/src/gui_menu_model.rs`
- `crates/gui-protocol/src/gui_menu_model/action_registry.rs`
- `crates/gui-render/src/render/scene.rs`
- `crates/gui-render/src/render/frame_preparation.rs`
- `crates/gui-render/src/render/shell_chrome.rs`
- `crates/gui-render/src/render/pane_chrome.rs`
- `specs/source_module_size_manifest.json`

S5A stays paused and unclaimed with its retained grant and unfinished evidence. No unrelated product rule, source-rights adoption, dependency, new numerical budget, scope reduction or evidence waiver. Claude exclusively owns protected prototype HTML; any genuinely uncovered visual state goes to the real owning lane. Existing failure guarantees and all six NPD steps remain. Commit owned changes locally on main with normal gates and substantive bodies. No remote publication, history rewrite, force push or hook bypass is added.

NPD-C01/C02 are complete on actual preparation/decision evidence; NPD-C03 is active. Runtime delivery, native proof, distinct replay and objective acceptance remain open.

Actual manager unchanged-behavior scope extension 01: the following existing context/admission consumers may represent genuine absence and retain loaded-origin safeguards:

- `crates/gui-app/src/terminal/runtime_terminal_control.rs`
- `crates/gui-app/src/terminal/runtime_terminal_profile.rs`
- `crates/gui-app/src/terminal/runtime_terminal_context.rs`
- `crates/gui-app/src/terminal/terminal_working_directory.rs`
- `crates/gui-app/src/terminal/terminal_accessibility_bridge/context_tests.rs`
- `crates/gui-app/src/application/production_status_refresh.rs`
- `crates/gui-app/src/terminal/terminal_session_controls.rs`
- `crates/gui-protocol/src/workspace_interaction/camera.rs`

No existing required terminal capability is waived; no PTY/parser redesign.

Actual manager unchanged-behavior scope extension 02 permits safe empty-registry admission at these existing boundaries:

- `crates/gui-app/src/terminal/terminal_session_render.rs`
- `crates/gui-app/src/terminal/terminal_session_lifecycle.rs`
- `crates/gui-app/src/terminal/terminal_session_interaction.rs`

Existing loaded-terminal behavior and return contracts retained; no fabricated active identity or PTY/parser expansion.

Actual manager scope extension 03 permits only the `specs/SPEC_PARITY.md` source-health inventory row reconciliation. Whole constructor ownership moves from protocol lib.rs to existing workspace_state: production 2940 → 2887, inline-test ceiling 1705 unchanged; all 66 debt metrics retained with no increase. No unrelated row or failed gate may be refreshed.

Actual manager scope extension 04 permits `crates/gui-app/src/terminal/terminal_accessibility_bridge.rs` to publish the existing Open dialog semantics through the current provider. Related protocol Open projection/tests already authorized. Path/value/focus/enabled/busy/refusal and removal on close must reflect actual state; no new layout/backend or model-only native PASS.

Actual manager scope extension 05 permits `crates/gui-app/src/application/workspace_keyboard.rs` to connect the already advertised Ctrl+N to `project.new.open`, and `crates/gui-render/src/render/shell_identity.rs` to omit only the unopened-state empty revision pill. Brand/loaded identity/clipping retained; no new layout/text/widget. The preparatory native shortcut failure remains retained as the reason for this correction.

## Historical C03 implementation and initial native checkpoint

<!-- EVIDENCE:NATIVE-PROJECT-DOORWAY:NPD-C03 -->

The existing Main now launches without a Project argument through an explicit
unopened workspace, absent request/terminal context and zero-session registry.
This display envelope has empty identities and no source/model/genesis authority;
loaded unbacked review scenes retain their established camera behavior. Existing
New/Open and shared Project admission remain the owners. Typed engine genesis
results enter the single existing Open worker, with origin/session generation,
writer/source/history admission and supervised terminal rejection retained.
Successful creation survives any subsequent Open refusal with explicit feedback.
Existing Open controls now publish path/focus/enabled/busy/refusal through the
current native accessibility provider. The advertised Ctrl+N reaches the same
New action as the menu. Unopened revision paint is omitted while the brand still
uses shared clipping; loaded identity paint is unchanged.

Twenty-one distinct focused Rust tests actually pass: startup protocol 3,
application startup and terminal Open protections 9, publication origin 1,
protocol Open/accessibility 6, keyboard dispatch 1, and expanded loaded/unopened
brand clipping 1. The source-health policy regression has 13 passing tests.
Focused structural drift passed; the same-source GUI/CLI/daemon build passed.
No dependency or protected prototype changed. Existing renderer dead-code warning
is disclosed; full workspace qualification is not claimed.

At that initial checkpoint, native GUI binary SHA-256 was
`0c1fb563e507bdc61a872af2e124409bbd77fbb731983a37df72b0c48c729059`;
base source is `1f9bef8d06c8a266a6b4afd126b28e044a955194` plus 34 exact
changed-Rust file pins retained in external `pre-r2-r3-native-source-pins.json`.
Native software input under X11/Vulkan has actually opened empty Main, reached
New with Ctrl+N, cancelled with no destination creation, created/opened Global A
(`17d3fbdd-4ec9-4f11-8fa7-a210afc4c4b7`) and explicit Factory B
(`08fa7b4c-5525-45dc-9748-c01c42d0c864`), and reopened A. GUI Save preserved
all four source documents and initialized the existing journal cursor to zero
applied transactions; this bookkeeping file is not an authored transaction.
Missing-directory/corrupt-source refusals retain the attempted path and prior
Project; real AT-SPI inspection exposes the dynamic refusal and control states.
The reader also emits an unsupported cache-interface warning, retained as a
limitation. These observations are software-native evidence, not physical/HUMAN
or an assistive-user trial. At that checkpoint complete T01–T09 correlation,
remaining negative/partial-outcome cases, replay and acceptance remained due.
The final C04 correlation below supersedes that intermediate pending state.

Raw logs/captures/fixtures remain task-owned and outside the repository at
`/home/bfadmin/Documents/datum-eda-workspaces/scratch/native-project-doorway`.
No images, build targets or raw logs are added to tracked proof. Preparatory
failed shortcut, early pre-paint capture and superseded pre-clipping build are
retained separately and never counted as final delivery PASS.

Actual manager scope extension 06 permits existing `terminal_accessibility_platform/atspi.rs` and `atspi_tests.rs` role corrections: native PANEL39/STATUS_BAR54/RADIO_BUTTON44, verified by installed GI. Current native captures expose incorrect older mappings; they are failed role evidence, not a PASS. Retain useful unchanged workflow/source observations, then inspect corrected native roles and independent replay. No provider redesign or new dependency. Native missing-rules derivative also exposed existing missing_required_shard admission gap; correction belongs to the already approved Open worker integrity boundary, not engine/schema policy.

Actual manager C04 scope extension 07 adds only `crates/engine/src/preferences/project_genesis.rs` and `project_genesis_lifecycle_tests.rs`. R4 same-repository native cached generation0 was accepted after a second GUI advanced the head to1; that observation is failed evidence. Validate live head/status once against the selected cached Global snapshot, including absent→present, before absent-destination genesis; keep values pinned after validation and exact replay/collision/optional-expectation rules. Expose existing read-only refresh for explicit Global New/Retry; never refresh/read Global for Factory. Preserve draft/focus and truthful current-context errors. Existing 700-line product_service owner stays unchanged. Current and newly affected genesis evidence must be checked at its actual scope before final acceptance.

## Actual C04 native correlation and frozen replay handoff

<!-- EVIDENCE:NATIVE-PROJECT-DOORWAY:NPD-C04 -->
The owner's crash/resume instruction retains the actual C02/scope01–07 grant.
The appointed manager recorded unchanged-scope continuation after verifying the
surviving source. The interrupted compilation has no result; completed earlier
checks were reused. Reboot removed temporary historical raw logs/receipts;
their past observations are not represented as currently available files.

Final base is `1f9bef8d06c8a266a6b4afd126b28e044a955194` plus 38 exact
Rust source hashes in external `final-native-source-pins.json`. Guarded GUI,
CLI and daemon build passed. GUI SHA-256 is
`d277ae53c8104a0dcf331ac9f59c1c01dd3d32d77195b35ee9f3cf5799bbd41b`.
Final native subject PID16928 and concurrent preferences writer PID17038
verified their running executable against that frozen hash, then exited0 with
complete input receipts. Native actions check focus ownership before/after;
final AT-SPI captures select the exact PID. All fixtures/captures/logs remain
external under `datum-eda-workspaces/scratch/native-project-doorway`.

External `native-proof-summary.json`, SHA-256
`e805c61d98497f83baaf8d46cc7bc673b09d8386c696d8df04bcb86f42e15b48`,
maps each actual scenario to individual artifact hashes and exact process/source
cohorts. It records reuse, failed attempts and limits rather than asserting that
every earlier run used the final binary.

| Scenario | Actual bounded result and correlation |
| --- | --- |
| T01 | Final no-argument Main, existing Ctrl+N/Open and absent Project; existing startup/origin regressions supplement native input. No sample/genesis or reserved Start page. |
| T02 | Native default Global/eight-value summary, explicit Factory and Cancel without a destination; final keyboard Tab/Shift-Tab/Enter and source-refresh focus captured. |
| T03 | Actual GUI-created A/B and final D. D UUID `1cddbd32-cff6-41ee-afea-e8db6002594d` opens through the existing worker; Save initializes the existing zero cursor. Original A is reopened after earlier controlled shutdown with stable UUID/source/history. |
| T04 | Same preferences repository changes generation1→2 in a second real GUI. Final New refuses the stale preview, preserves name/location/Create focus and leaves D absent. Explicit Re-read shows generation2/Imperial, then ordinary Create/Open succeeds with that immutable receipt. Earlier native unreadable/migration refusals and explicit Factory selection preserve source; existing-destination collision refuses without overwrite. |
| T05 | Native pending Open Return/Escape retains prior C and removes dialog nodes. Existing exact-origin, supersession, publication and terminal rejection regression proof is reused at unchanged code. |
| T06 | Native missing directory, corrupt source, unsupported schema and held-writer refusals preserve prior Project/source. Releasing the owned writer permits retry. Missing required rules now refuses through the existing integrity boundary, with focused R3 proof. |
| T07 | Final ordinary Open consumes a genuine committed canonical stage and restores its exact head without appending the journal. Corrupt-head Open refuses, preserves corrupt bytes and retains the recovered Project. Earlier actual created C/subsequent terminal-Open failure preserves C; later ordinary Open succeeds. |
| T08 | Actual X11 software keyboard, focus and controlled window close; native GI roles/states/full refusal values, including corrected roles and final PID attribution. Narrow brand clipping regression is retained. No physical/HUMAN or assistive-user acceptance is claimed. |
| T09 | With Global Imperial generation2, ordinary reopen/Project Preferences displays A's original Metric units. A/B/D source and history hashes remain identical; inspection/cancel adds no design transaction. |

Shipped Ready Units values have eight valid literal defaults and one user
layer; a missing effective Units value is not a normally reachable repository
state. A clearly labelled test-only selected-descriptor fault establishes a real
Ready snapshot with one absent default and proves typed refusal, absent destination
and byte-preserved repository. That focused test passed1/1. It is defensive unit
proof, not fabricated native input. The app's separate factory-only-reader test
passed1/1 and proves unavailable-provider draft preservation/explicit Factory,
not unresolved values in an otherwise available snapshot. Actual native migration
and unreadable preconditions remain separately recorded.

Affected genesis batch is exactly9 PASS/1 FAIL; do not call the family green.
The failing concurrent-identical-Factory test observed RepositoryIo/live-writer
at the main caller; publication/replay winner/loser interleaving is inferred.
Distinct review verified the original test and relevant Factory suffix/resolver/
lease byte-equal to HEAD; Factory skips R4 Global validation. The appointed manager
classified this preexisting defect as related-only intake
`dat-genesis-concurrent-replay-lock-sg3e`, retained open. No lock bypass, waiver,
retry-until-green or new prerequisite. Earlier focused startup/Open/accessibility,
R2/R3 and rendering results retain their actual scopes; no broad1042-test rerun.

The full drift attempt remains exit101 at16 preexisting Clippy errors in eight
unchanged engine files; later full gates were unreached. It is not a global PASS.
Native AT-SPI GetItems cache warnings remain captured in
`dat-atspi-cache-introspection-4v0p`; dynamic PID-qualified properties succeed.
Failed R1 clipping/R2 role/R3 shard/R4 stale observations and prepaint/argument
mistakes are retained and distinguished from corrected outcomes. Broader
qualification, S5A and successor/publication boundaries remain unchanged.

Actual distinct source review found no actionable production defect in R4 and
no finding in the injected test delta; it is source review only. Source is now
frozen; no build or GUI process survives the handoff. The appointed manager
received the actual results, reserved desktop ownership to the distinct reviewer
and directed C04 completion/C05 activation. This marker does not supply the still
then-required C05 native replay or C06 objective disposition.

## Actual C05 independent source and native replay

<!-- EVIDENCE:NATIVE-PROJECT-DOORWAY:NPD-C05 -->
Distinct reviewer `/root/reorganization_review`, thread
`01a13e3f-8074-7943-91c1-8a0b09fc21e8`, returned actual **bounded PASS**
for the frozen ordinary native Project doorway. The appointed manager received
the receipt. External `c05-independent-review.json` SHA-256 is
`0dadb2e990ecac79aa4414063e7834877f767a9c26513ef229c4f25c9f09c804`.
All 38 changed Rust pins and 53 supporting artifacts were independently checked.
Both review runs used exact GUI `d277ae53…`, exited0 with complete input and
drained-device receipts, and released the desktop without repository edits/builds.

Actual distinct replay covers empty startup/disabled Project actions, native
New/Cancel/no destination, Global A and explicitly selected Factory B creation,
Save/A→B→A, Open/Cancel/removal, existing-destination refusal preserving draft
and Create focus, missing-required-rules refusal preserving the derivative/prior
Project, native focus/roles/values, 480×360 Main and fresh-process disk reopening.
Independent A UUID is `175761e2-d709-42b7-812a-a01461a58ade`; B UUID is
`f68c6c99-35a1-47bf-82c4-12bc02b74416`. Their authored files and journal/revision
authority remain byte-identical after the exercised refusals/reopens. Expected
terminal operational-file changes are distinguished from authored/history state.

The reviewer also assessed the exact current stale-generation/refresh/receipt,
eligible committed recovery/corrupt refusal, partial outcome and copy-once
evidence at its declared source cohorts. This is not a claim that every worker
scenario was independently rerun. An unsuccessful rapid synthetic Tab sequence
is retained; isolated Tab/Shift-Tab traversal reached each expected control and
confirmed Create activation. No source defect was reproduced or code correction
requested. Native AT-SPI cache warnings, lack of physical/HUMAN/assistive-user
acceptance, actual genesis9 PASS/1 FAIL and full-drift/qualification limits remain.

The appointed manager directed a normal local implementation commit with C05
complete and C06 pending/selected, followed by the fresh exact C06 packet and
actual objective disposition. This receipt supplies C05 only; no C06 acceptance,
issue closure, successor execution or remote publication is inferred.

## Actual C06 objective acceptance and landing

<!-- EVIDENCE:NATIVE-PROJECT-DOORWAY:NPD-C06 -->
Appointed manager `codex-reorganization-manager-20261014` recorded actual
objective acceptance under the owner appointment/clarification and PM059:

> NATIVE-PROJECT-DOORWAY: approve DOORWAY-ACCEPTANCE — implementation 45f77006c661019e02928b90dba2a9f1b177bc4d; exact C04 native matrix and actual distinct C05 bounded PASS, under retained NPD-MANAGED-01 scope.

The durable external `manager-c06-acceptance.json` SHA-256 is
`6bddeb00e9bf193b3152b141c86560b09a5f1a1db7be40b2fdfe7faad6664895`.
It verifies the exact fresh C06 packet, C04 map, actual C05 receipt, all38 Rust
pins and clean48-path implementation commit. This is actual delegated manager
input, not an invented owner reply, HUMAN judgment or full-qualification PASS.

Implementation `45f77006c661019e02928b90dba2a9f1b177bc4d` passed normal
file-lane, temporary-artifact and staged-rustfmt hooks. The earlier C06 metadata
transition initially retained an execution claim; render refused it. It was
corrected to specified/owner_decision, open/unassigned and no claim before the
implementation commit, without relaxing the validator. C06 acceptance followed
that commit and exact packet; closure does not infer approval from implementation.

C01–06 are complete, the issue closes against that implementation, and bounded
development availability reflects the ordinary native workflow above. All actual
Clippy/concurrent-genesis/cache warnings, rapid synthetic input observations and
physical/HUMAN/assistive-user/platform/authoring qualification limits remain.
No additional compilation or native replay is justified by metadata-only landing.

The manager's separate explicit selection disposition, external
`manager-closure-selection.json` SHA-256
`35637b4fd9d264a01ec205e97d25ba34da6c37065c22189a8edb77be0d721222`,
restores only pre-NPD `UVT-S5A-BUILD` selection while landing/unselecting NPD.
S5A retains its exact prior state, grant, dependencies, unfinished C04 contract,
evidence, tracker/assignee and unclaimed state. Its owner pause remains in force;
no S5A session/work resumes and no successor execution is authorized. No source
change, new dependency, waiver, protected prototype edit or remote publication.

The existing development-availability validator requires a portable typed
ordinary-application record; a Markdown evidence path was rejected during
closure. The manager explicitly authorized the compact
[availability record](native-project-doorway.json) in this existing evidence
family. It maps the actually approved workflow/safety outcomes to the exact
implementation, C04/C05/C06 receipts and retained limits; no raw captures/logs
or new proof schema/framework. Closure validation also required the completed
step pointer to be null and removal of the now-closed NPD issue from the
foundation's derived pending-unblocks list. The actual dependency edge remains;
these rejected intermediate states were corrected without validator changes.


<!-- REQ:NATIVE-PROJECT-DOORWAY:NPD-C07 -->
## NPD-C07 — owner-reported startup QA repair

The owner reported failed ordinary New Project creation with a `~/` Location,
misleading Global Units failure text and a blank terminal after cancelling New.
The owner then explicitly instructed the appointed manager: “this should have
been sent to you. I need you to correct these issues for me to QA test this project”,
and “talk to the peer agents.” These are actual messages in manager thread
`01a13df1-7a0d-72c1-bbdd-f0d96d481598`. The session is now in Default mode.
This is an authorized correction to the existing doorway, not successor feature
selection. Historical C01–06 evidence stays at its original source and scope;
the owner-observed failure overrides an unqualified current-readiness claim.

<!-- APPOINTMENT:NATIVE-PROJECT-DOORWAY:QA-REPAIR -->
{"owner_appointed":true,"session":"codex-production-manager-20261016","task_keys":["NATIVE-PROJECT-DOORWAY"],"decision_kinds":["review_provisioning","review_receipt","continuation","objective_acceptance"]}

Existing full-destination semantics remain: Location is the new Project's entire
path, separate from its display name. The governed New form shows leading `~/`;
resolve that shorthand at the GUI boundary without shell evaluation, general
variable/other-user expansion, implicit parent creation or engine publication
changes. Never append the display name or overwrite an existing destination.

Keep a valid pinned Units source and eight-row summary after destination or
publication errors; show the actual reason. Editing the relevant input must
permit correction and retry without unrelated Global rereads. Genuine stale,
unreadable or incomplete seed failures still invalidate readiness and require
explicit source resolution; never select Factory implicitly. In the historical
C07 scope, the unopened terminal had no shell or Project context and had to say so through existing
status/chrome, without a fabricated session, new welcome surface or terminal
implementation change. NPD-C08 supersedes that no-shell restriction under the
owner's subsequent direction while retaining truthful absent Project authority.

Dev-agent session `01a14799-fef3-7e01-b079-2fcae52aa45d` owns the source/test paths
below. Manager `codex-production-manager-20261016` owns state, factual records
and acceptance. The single synchronized lease records both disjoint assignments.
Independent reviewer `/root/closeout_review` remains read-only and may not accept
its own implementation. Manager pulls peer results directly; the owner is not
a message relay. Source release follows actual route readiness, not this entry.

Implementer paths:
- `crates/gui-app/src/windows/global_preferences_runtime/new_project.rs`
- `crates/gui-app/src/windows/global_preferences_runtime/new_project/tests.rs`
- `crates/gui-protocol/src/new_project.rs`
- `crates/gui-render/src/new_project_dialog.rs`
- `crates/gui-protocol/src/workspace_startup.rs`
- `crates/gui-protocol/src/workspace_startup_tests.rs`
- `crates/gui-app/src/application/native_project_startup_tests.rs`
- `crates/gui-render/src/terminal_tab_strip_tests.rs`
- `scripts/prove_native_project_startup_gui.py`
- `scripts/check_global_preferences_boundary.py`
- `scripts/test_global_preferences_boundary.py`
- `.datum/native-project-startup-repair`

Manager paths:
- `README.md`
- `docs/reviews/workflow-delivery-rollout/native-project-doorway.md`
- `docs/reviews/workflow-delivery-rollout/native-project-doorway.json`
- `specs/active_frontier.json`
- `specs/PROGRESS.md`
- `.beads/issues.jsonl`
- `specs/spec_governance_manifest.json`
- `specs/evidence_traceability_manifest.json`
- `specs/source_module_size_manifest.json`
- `docs/SOURCE_HEALTH_POLICY.md`
- `specs/spec_parity_manifest.json`

Acceptance requires focused path/refusal/retry/source-readiness regressions and
an ordinary no-argument GUI run using real App/window event dispatch or desktop
input, not direct coordinator/genesis or input-handler calls. In owned scratch,
exercise cancelled New and truthful no-session terminal; an occupied/missing-
parent destination with byte-preserving refusal; keyboard correction to a fresh
`~/.../<project>` without a Global reread; successful Create/Open with eight-item
immutable Units receipt, a live Project terminal, Save and fresh reopen.
Reach Library Browser and Package creation on that GUI-created Project, rather
than bootstrapping the starting Project in a helper. Inspect actual refusal,
recovery and terminal frames and focus/readiness/accessibility state. Preserve
absolute/relative destination behavior, stale/unreadable source protection,
explicit Factory isolation and created-but-unopened recovery.

Use only `.datum/native-project-startup-repair` for local disposable inputs,
logs and captures; no external workspace collection or owner Project mutation.
Reuse adequate unaffected proof with exact limits, retain failed attempts and
source/binary bindings, run required source-health/dependency/drift gates, obtain
actual independent source/runtime/rendered and factual-documentation review,
land the corrected source and source-pinned availability, then close this repair.
No physical/HUMAN or full-qualification result is inferred. S5A stays held.

Activation: source-health baseline passes for 3,188 files at `746af44812275bb2cb809cdd1f4241a66e321be4`.
Current result: C07 accepted at `0a8e8b5db1547f4c8d62983f397db99cc4b76fbc`; see the current result and retained qualification limits below.


C07 source release: actual independent `/root/closeout_review` readiness PASS,
no mandatory findings, at amended-record SHA-256
`0d689512e1632b8c9d14ada59f787ed2ff7a9d6c6cfc7142bfe1bd0c2ffaa2d2`.
Its review accounts for 150 unique members across rollout (28),
Workspace/Preferences (80), manual foundation (4), terminal research (4),
terminal prototype (2) and editor shell/panes (35), using adequate complete
accepted-corpus reuse, exact comparisons and changed-consumer assessment.
No fresh whole-corpus or runtime PASS is claimed. Assignment/claim parity,
historical C01–06, withheld current readiness and S5A preservation were verified.
The appointed manager accepts this actual receipt and releases the documented
source work under the owner-authorized C07 repair; implementation, ordinary GUI
proof, independent source/runtime/rendered review and final acceptance remain.

C07 unchanged-scope correction: the extracted New Project tests exposed an
existing lexical boundary-checker defect: `tests.rs` was scanned as production,
and the old production inventory included a type name used only in inline
tests. The manager authorizes the checker and its focused regression within
this repair to classify tests consistently and retain the exact real production
service-owner inventory. This does not grant a new Preferences adapter, private
writer, dependency or product mechanism. The synchronized lease includes these
two bounded paths; implementation remains with dev-agent.

C07 QA-platform clarification from the owner: a forced-X11 run is not the
primary proof for the owner's normal Wayland application. The observed ordinary
launch reports `window_backend=wayland`, `gpu_backend=Vulkan`, Intel HD Graphics
P630 and Mesa 25.0.7. WGPU is the renderer; Wayland/X11 identify window and input
transport. Use the same `target/debug/datum-gui` build and ordinary no-argument
launch with the default terminal profile. Record the actual window, GPU and
compositor identities. Any X11 compatibility or nested-compositor evidence must
be named separately and must not claim native KWin input qualification. No
private UI actions, Project bootstrap, dependency installation or permission
bypass may replace the ordinary New/correction/Create/Open workflow.

The owner explicitly clarified: “datum was designed for wayland/wgpu not x11”.
Native Wayland plus WGPU is therefore the required product and primary QA target
for C07. X11 results cannot establish that readiness. A nested-compositor run
must keep Datum itself on Wayland/WGPU and identify its compositor limits; it
does not become an ordinary KWin workflow by relabelling.

The manager also owns a bounded README launch correction: identify the native
Wayland/WGPU QA entry, build GUI/CLI/daemon companions together and distinguish
the interactive launch from automated proof scripts. This addresses the owner's
reported launch ambiguity without creating another launcher or test framework.


<!-- EVIDENCE:NATIVE-PROJECT-DOORWAY:NPD-C07 -->
## C07 delivered result and manager acceptance

Source landed at `0a8e8b5db1547f4c8d62983f397db99cc4b76fbc`, exactly the
11 independently reviewed source hashes. Dev-agent explicitly released source
ownership. Leading home shorthand resolves at the GUI boundary. Destination
errors retain the pinned eight Units and allow correction/retry; actual seed
failures remain blocked. Existing terminal chrome truthfully identifies the
unopened state. Engine publication and full-destination semantics are unchanged.

Actual `/root/closeout_review` bounded source/runtime/rendered **PASS** has no
remaining mandatory findings. The reviewer verified source catalogue
`3c67390522115862432165e12fe92dcb26962e97164f73b606f7118eb523663d`,
evidence catalogue
`7eebcb986750a33ae58f7bc543c4ba933f4fa2a7619fdc4f3426339e0db492e5`,
all 11 source pins, 26 evidence hashes and matching executable identities.
The GUI SHA-256 is
`158b89b3151eb275d507177502275ab5cbe08ffcfbd0ca18bb4a01d98933068a`.
The existing companion JSON records exact bindings, outcomes and reproduction.

Ordinary bare Wayland/WGPU/Vulkan startup, cancelled New/no shell, occupied
home-path refusal with preservation, keyboard correction without Global reread,
Create/Open with the eight-item receipt, live Project terminal, Save and fresh
Open are observed. The reviewer inspected the actual rendered states. A targeted
ordinary Open of that same GUI-created Project then reached the actual Create
Package draft. Its terminal and metadata fields are visibly present; draft and
Cancel preserve the original canonical source/journal/revision six-file map.

The same binary launches normally on KWin Wayland. Full workflow input used an
owned nested Weston compositor: xdotool drove its outer X11 window while Datum
itself remained a native Wayland/Vulkan client. Initial X11 backing-pixmap
captures were stale; direct `weston-screenshooter` capture on the disposable
compositor supplied the actual final Package frame. Rejected early frames remain
failed evidence, even where a script's executed-state checks said PASS. No owner
desktop permission or configuration was changed.

Twenty-one focused Rust tests pass; two existing GPU tests remain ignored.
Four boundary-checker regressions and 13 source-health policy tests pass.
Matching GUI/CLI/daemon build, source-health (3,191 files), dependency authority,
formatting and the focused structural gates pass. Full drift still stops at the
same 16 Clippy errors in eight unchanged engine files (`dat-lk0l`); subsequent
full gates were not reached. This is not a full-workspace qualification PASS.

Current teardown is SIGTERM followed by ordinary fresh-process Open. Historical
controlled Close evidence at `45f77006` remains reusable only because independent
review traced and byte-compared the unchanged close/drain/receipt owners and
close-after-submit test. Historical invalid/busy/corrupt/recovery and complete
keyboard/accessibility evidence likewise keeps its original source and scope;
current affected behavior is supported by the new focused and ordinary tests.
No new KWin input/Close, physical/HUMAN, assistive-user or full-platform acceptance
is claimed. Existing Package creation/history/Footprint evidence stays scoped to
`1459e21f`; this repair proves current startup-to-Package-draft reachability.

Appointed manager `codex-production-manager-20261016` accepts the actual review
receipt and objectively accepts NPD-C07 under the recorded owner appointment and
PM059. Restore bounded Native Project and Package development availability, close
`dat-native-project-startup-vrf` with the source landing, release the lease and
retain all historical receipts and qualification limits. S5A stays held. This
is delegated acceptance, not a fabricated owner/HUMAN disposition.

<!-- SELECTION:NATIVE-PROJECT-DOORWAY:NPD-C07-DELIVERED -->
The manager explicitly selects idle after delivery of this repair. No successor
or S5A execution is selected or authorized. The owner can start a fresh QA
instance with `env -u DISPLAY ./target/debug/datum-gui`; Location must name a
complete new folder, for example `~/Documents/datum_projects/qa-project-01`.
`README.md` is the interactive build/launch entry. The tracked proof runner can
recreate its own disposable inputs inside `.datum/native-project-startup-repair`;
retired external collections and untracked helper scripts are not required.


<!-- REQ:NATIVE-PROJECT-DOORWAY:NPD-C08 -->
## C08 — terminal availability before Project Open

The owner reported that withholding the terminal prevents CLI Project creation
and starting an AI agent to help create or load Projects. This direction in
manager thread `01a13df1-7a0d-72c1-bbdd-f0d96d481598` authorizes a real terminal
before Project Open. It amends C07's no-shell behavior, preserving C01–07 as
historical delivery at their original sources. PM005/027 and native terminal
section 4.1 now distinguish a real session from optional Project authority.
PM028 still requires a real scoped Project for Project-integrated MCP claims.

The existing owner-appointed `codex-production-manager-20261016` coordinates
C08 under the retained task appointment and PM059: source assignment, actual
independent review, unchanged-scope correction and objective acceptance. The
owner is not a routine approval or peer-message relay. The synchronized lease
records root-owned authority/state paths and dev-agent-owned implementation.

Dev-agent `01a14799-fef3-7e01-b079-2fcae52aa45d` owns the bounded terminal session,
process, context, spawn, lifecycle, control, profile and cwd changes under
`crates/gui-app/src/terminal/`, their tests, the existing native startup owner
and tests, `application/production_status_refresh.rs`, protocol workspace
startup/status and tests, terminal tab-strip tests and the existing
`scripts/prove_native_project_startup_gui.py` ordinary proof runner. This is
permission for the necessary unbound-session capability, not unrelated terminal
work. Extract cohesive responsibilities under PM022; no source-size exceptions
or third-party terminal dependency. Root owns this record, PM005/027, native
terminal spec, evidence manifests, Frontier, generated Progress and beads.
Independent `/root/closeout_review` remains read-only.

Acceptance:

- No-argument native Wayland/WGPU Main exposes a working real shell before any
  Project. Normal shell commands, the canonical CLI and installed agent
  executables work; focus remains governed by the existing explicit input owner.
- Session identity and actual cwd remain truthful. Strip stale inherited Datum
  Project/context/discovery/credential values without removing ordinary agent
  authentication. Do not synthesize a Project root or write Project sidecars
  into the launch directory. Unbound runtime resources have owned cleanup.
- Create an owned Project through canonical CLI/engine operations, then open it
  through ordinary GUI controls. Preserve the original PTY, child process,
  environment, cwd, shell state and scrollback. Reuse existing appended Project
  terminal publication; never silently bind the original session or its agents.
- Exercise Cancel, refused/superseded Open, tabs/splits, restart, exit and guarded
  shutdown. Preserve process and credential ownership, truthful error/status
  feedback, Project-bound behavior and the accepted Units/refusal protections.
- Obtain focused behavioral regressions, actual rendered/input Wayland proof,
  source/binary binding and independent review before objective acceptance and
  delivery of a new pinned QA target. Direct agent launch does not establish
  full MCP interoperability; historical qualification remains scoped as recorded.

Use existing `.datum/native-project-startup-repair` ignored scratch for disposable
inputs, logs and captures; builds remain disk-backed guarded targets. Never use
or modify the owner's `~/Documents/datum_projects` or retired external scratch.
No new GUI command bridge, terminal-output mutation route, dependency, prototype
change, global agent configuration, unrelated feature or S5A work is authorized.

Actual authority preflight: `/root/closeout_review` found no unresolved owner
choice and confirmed PM005 session persistence, PM024/027 real-shell behavior,
PM028 direct-agent permission and genuinely bound Project credentials. It reused
the accepted 150-member six-route assessment with exact corpus comparisons,
reviewed the full terminal-focus incident route and PM005/024/027/028 plus the
agent interoperability spec. Current route changes require changed-consumer
review and deliberate digest reconciliation before source release. No new
implementation, runtime or full-qualification PASS follows from this preflight.


<!-- EVIDENCE:NATIVE-PROJECT-DOORWAY:NPD-C08 -->
## C08 delivered result and manager acceptance

Source landed at `db81f73442dc5f33c1d125a4e40de5b7485fca60`, exactly the
30 reviewed source pins. Dev-agent released source ownership. Bare Main starts
a real shell in the actual launch cwd, with Home/root fallback only if that cwd
is unavailable. Session identity and private runtime are independent of Project
authority. Stale inherited Project, discovery, credentials, sockets, MCP and
agent provenance are cleared; ordinary authentication and adapter settings remain.
Missing CLI companion reports its limitation without withholding the shell.

Ordinary CLI creation followed by GUI Open appends a genuinely bound Project
terminal and preserves the original shell, child, cwd, environment, state and
scrollback. Tabs, splits and restart retain their origin. Owned runtime survives
asynchronous session removal until the supervisor verifies process closure.
No new command bridge, private writer, dependency or prototype change is added.

Actual `/root/closeout_review` **bounded source/runtime/rendered PASS** has no
remaining mandatory findings. It independently verified source catalogue
`8c3ff2f98ffcc08062e3d6d971f0b0889f3efe6608073f7a481eee7dab011612`,
evidence catalogue
`0dbf7271a3165e69c01296ac9bbbc3a856ee722acacd94087137dba57bedb069`,
all 30 landed sources, 71 evidence hashes and all 16 direct Wayland frames.
The companion JSON retains exact source/binary bindings and reproduction.

Accepted ordinary evidence covers working pre-Project shell, matching CLI,
installed Codex executable/version, New/Cancel, invalid Open, canonical
human_cli/Factory creation with eight Units, GUI Open and fresh bound discovery.
The same original shell and child survive, and ordinary shell exits verify
owned child/PTY/runtime cleanup. Five canonical source/revision hashes remain
unchanged; no Project sidecars appear in the launch cwd. The same GUI runs on
KWin Wayland/Vulkan. The automated workflow uses owned nested Weston with outer
X11 input; Datum itself is a native Wayland/WGPU/Vulkan client throughout.

Accepted focused cohorts contain 99 passing executions covering 97 distinct
binary/test names, zero failures and five existing ignored tests. Final real-shell
and governed-agent PTY tests cover the last provenance correction; earlier
adequate session/context/cwd/unopened checks are explicitly reused. Matching
build, dependency/resource/transport, 25 boundary regressions, scoped formatting
and source health (3,193 files) pass. Actual full drift stops at the same 16
preexisting engine Clippy errors (`dat-lk0l`); later full gates remain unreached.

The scrolled after-Open frame proves preserved body/position only: its tab/focus
chrome and accessibility text are stale. The later actual shell-input frame
shows both tabs, Terminal focus and correct original process/environment.
No cause or complete visual/accessibility qualification is inferred. The
preexisting scroll-only accessibility refresh omission is separate intake
`dat-terminal-scrollback-a11y-hnti`. Failed compile/probe/focus attempts remain
failed evidence; the final runner records the precise observation limits.

Installed-agent version execution and generic-child preservation are not a
model task or full MCP qualification. KWin supplies launch smoke only. Current
ordinary terminal exits are not a new ordinary GUI Close run; historical Close
supports unchanged dispatch/coordination paths, combined with current affected
cleanup tests. Changed supervisor/status owners are not declared byte-identical.
C01-C07 refusal, Units, Create/Save/reopen and Package evidence remains historical
at its exact source, with no relabelled full current matrix or HUMAN result.

Appointed manager `codex-production-manager-20261016` accepts the actual review
receipt and objectively accepts NPD-C08 under PM059 and the recorded owner
direction. Close `dat-native-project-startup-vrf` at the source landing, release
the lease and restore bounded Native Project development availability. All
retained qualification and S5A boundaries remain; no successor is authorized.

The new QA copy is `target/qa/terminal-db81f734/debug/datum-gui`, with matching
CLI/daemon, actual dependency records and `../build-info.json`. These are
independent copies of the reviewed binaries, with original timestamps preserved.
The older owner's QA instance remains intact. To start the unbound shell in Home:

```bash
cd ~
env -u DISPLAY -u WINIT_UNIX_BACKEND /home/bfadmin/Documents/datum-eda/target/qa/terminal-db81f734/debug/datum-gui
```

The owner's optional post-create terminal handoff/Home-default idea is retained
as intake `dat-terminal-project-handoff-emzi`. Distinguish no session, idle shell
and a running agent/job; changing cwd alone cannot rebind credentials or a live
process. No prompt, automatic migration or new default cwd is ratified here.

<!-- SELECTION:NATIVE-PROJECT-DOORWAY:NPD-C08-DELIVERED -->
The manager explicitly selects idle after this delivered repair. No successor
or S5A execution is selected or authorized. Raw logs, inputs and captures remain
ignored task scratch; the existing receipt and reproducible runner carry the
necessary provenance without new tracked audit dossiers or external collections.


## NPD-C09 — Browse for New and Open Project

<!-- AUTHORITY:NATIVE-PROJECT-DOORWAY:NPD-C09 -->
<!-- REQ:NATIVE-PROJECT-DOORWAY:NPD-C09 -->

Owner QA on 2026-10-17 confirms the launch terminal is accessible and reports:
“both new project and open project is missing a browse button. this is standr
operating procedure for any application.” This directs the bounded Browse
correction under the existing startup manager appointment. After testing the
uncommitted Browse candidate, the owner requested the complete process in a
single dialog instead of menu diving and explicitly approved proceeding with
inline navigation, a visible name-derived folder default and an optional
same-dialog override. This refinement supersedes the initial C09
Browse/Select/return interaction and its prohibition on name-derived folders.
C01-C08 acceptance remains at its recorded source; it is not evidence of C09
delivery. The earlier C09 QA candidate is historical evidence of its own scope.

The appointed manager selects one shared first-party folder browser inside the
existing New/Open native window and Wayland/wgpu surface. It uses the existing
approved runtime and standard filesystem interfaces. PM063 adds a typed host
New Folder/Rename service; no new dependency, portal helper or private engine
writer is introduced. Directory reads
run outside the event thread, stale completions cannot change a closed/replaced
form, and only visible list rows are rendered. The integrated dialogs and shared
navigator must have matching pointer, keyboard and accessibility behavior.

New contains Project name, **Location** (existing parent) with **Browse…**,
**Folder name**, **Use project name** and the complete destination preview in
one dialog. Units initialize automatically from Global defaults; no Units
choice, eight-value panel, source/generation metadata or related focus stops
appear in New. Configuration remains in the existing Global/Project Preferences
Units pages. Passive Browse is read-only; PM063 governs explicit host actions.
The displayed current directory
is the actual parent; a highlighted row is navigation selection, and Enter or
double-click enters it. This distinction is visible. Use project name defaults
checked and visibly derives the folder name. Unchecking enables an explicit
same-dialog override; checking restores derivation. No implicit sanitization or
trimming changes path identity. Invalid components require a corrected name or
override. The exact displayed parent/folder combination is the complete
destination submitted to the existing engine; the parent field never silently
reuses the former complete-path Location meaning. The engine's genesis schema,
atomic publication and absent-destination rule are unchanged.

Open contains the same navigator and shows the selected directory, or the
current directory when none is selected, as its full candidate. Open invokes
existing validation and admission directly. Neither action has a separate
Browse page, Select/return step or extra confirmation. Passive browsing creates nothing.
Create and Open remain the sole Project publication actions. Navigation preserves the
draft, explicit folder override and automatically pinned Global source. Cancel closes
the action without changing the current Project or terminal sessions. Errors
remain correctable; no failed or cancelled navigation clears valid Units,
selects a fallback or starts an operation.

The inline navigator provides path navigation, Up/Home, a scrollable folder list
and truthful loading/empty/unreadable states. The sole form footer is Cancel and
Create or Open. Typed paths remain available. Include dot-folders; introduce no
hidden-folder or remembered-location preference. The default 760-by-720 and
minimum 620-by-620 native window must keep the final action reachable and the
complete destination and errors inspectable through shared scrolling where needed.
Directory/leaf errors are actionable; unreadable, disappearing,
symlinked and large directories require bounded handling without event-thread
blocking. Passive browsing reads paths only; it never follows a folder as Project
mutation authority. No Start page, recents, template, preference or S5A expansion.

### Owner QA correction: automatic Units initialization

<!-- AUTHORITY:NATIVE-PROJECT-DOORWAY:NPD-C09-AUTO-UNITS -->

On 2026-10-17, after reviewing the integrated dialog, the owner questioned why
Units configuration was part of creation. The proposed correction was to remove
both Units radios and the eight-value panel, initialize from Global defaults
automatically and leave configuration in Project Preferences. The owner confirmed
“this is exactly what im saying.” and clarified: “units can be configured at any
time from either the global or project prefrences anyway.
edit->prefrences-global/project->units”. This is actual owner direction for the
bounded presentation correction, not a delegated invention of a product rule.

The existing snapshot-at-open, complete eight-value validation, expected-generation
check and immutable genesis receipt remain. No-file-yet resolves the ordinary
Global defaults-only contribution. Stale, unreadable, incomplete or
migration-required defaults preserve the draft, show the actual error and block
Create; only that failure state offers **Re-read Global defaults**. Successful
reread restores readiness and never creates automatically. Cancel permits resolving
the reported source problem through its existing governed recovery mechanism
before reopening New; no new GUI store-repair or migration action is implied.
New has no Factory selection or fallback.
Destination failures retain the valid snapshot and remain correctable. Later
Global changes never alter an existing Project; Project Preferences owns its
journaled Units changes without geometry rescaling.

The same manager coordinates the retained C09 work: actual Claude reconciles
only the protected visual target; dev-agent removes retired native controls,
state, hit/focus/accessibility targets and updates affected proof; independent
review verifies source, normal and failure states and a new native Wayland QA
candidate. The previously accepted `a42d6eb5` source, binaries and evidence stay
frozen and establish only their historical scope. No new engine API, dependency,
Open/terminal behavior, acceptance waiver or successor grant is introduced.
Protected visual-owner landing remains unresolved. This correction supersedes
only the earlier current C09 Units-review presentation requirements.

### Owner QA correction: visible Browse and direct path entry

<!-- AUTHORITY:NATIVE-PROJECT-DOORWAY:NPD-C09-BROWSE-AFFORDANCE -->

After testing the automatic-Units candidate, the owner reported that Browse was
gone and required an easy, conventional human workflow. This corrects the
always-visible folder list presentation: **Location** and **Browse…** are
visible on both New and Open. Browse reveals the existing Up/Home/list controls
in the same dialog and focuses the list; repeated Browse refocuses it. There
is no second picker page, Select/return action or new confirmation. Hidden
navigation is excluded from pointer hits, Tab order and accessibility nodes.
The existing window sizing and optional folder-name override remain; the
ordinary New path needs only a name, parent location and Create.

Typed Location must be validated for Create/Open without a hidden mandatory
Enter or Browse action. Existing asynchronous reads and stale-answer rejection
must prevent acting on edited, closed or replaced form state. New Location is
the parent, while the exact resulting Project directory remains visible; Open
retains its selected/current directory candidate and engine admission. Navigation
publishes nothing. Automatic Global Units, error-only reread, cancellation,
source safety and terminal preservation remain unchanged.

The existing C09 manager coordinates the actual Claude visual correction,
dev-agent implementation and independent review. The frozen `6183c81e` source,
QA binaries and evidence remain historical at their original scope; they do
not establish acceptance of this corrected interaction. No new dependency,
engine/terminal behavior, publication path, successor grant or gate waiver.

Actual independent `/root/closeout_review` returned
**BOUNDED_PROTOTYPE_AND_AUTHORITY_PASS** for corrected target
`9cc844bf9e21d0bd651e731e6bc58526a7abfbb14c6c3798623593525d960030`.
The manager adopts it for bounded C09 native implementation. All 19 rendered
states are covered by 22 corrected captures; the final outside-frame wording
change preserves every frame and CSS block exactly. The normal expanded New
form keeps the complete destination visible. Listing is discovery only, Home
is explicit, and Open retains its valid selected-child candidate. This releases
implementation and focused proof; native/runtime acceptance remains pending.

### Owner QA correction: reusable browser actions

<!-- AUTHORITY:NATIVE-PROJECT-DOORWAY:NPD-C09-FILE-ACTIONS -->

The owner requested right-click New Folder and Rename in the explorer pane,
then required building shared objects once for reuse by future menus and said
“please continue.” This authorizes the bounded shared implementation in
[PM063](../../decisions/PRODUCT_MECHANICS_063_SHARED_FILE_BROWSER_ACTIONS.md),
including ordinary text editing in the New/Open fields. It extends the retained
C09 assignment; it does not require another owner approval of this same request.

New and Open must consume one context-menu control, one naming editor and one
typed host-filesystem service. Host actions preserve canonical Project/library
storage and live session anchors, refuse overwrite/relocation and retain actual
completion after dialog cancellation. Passive navigation and cancelled naming
drafts write nothing. Unrelated menu migrations and new dependencies are excluded.

The actual Claude-owned visual amendment and independent review precede source
release. At this checkpoint implementation, native proof and new QA delivery
remain pending. The accepted b015 source, binaries and receipts stay frozen;
their PASS does not cover this extension. The protected visual-owner landing
blocker, required route reconciliation, full qualification and S5A hold remain.

### Shared-actions authority review and nonvisual source release

<!-- DISPOSITION:NATIVE-PROJECT-DOORWAY:NPD-C09-FILE-ACTIONS-AUTHORITY -->

Actual independent `/root/closeout_review` returned
`BOUNDED_AUTHORITY_AMENDMENT_PASS`, with no remaining mandatory findings, for
PM063 at `a94ac965d7c9ae41fe69352b6ba799eab56bc58d4ef37ea76ed429d38c7bf806`
and the four amended consumers. The review covered146 unique route members:
141 unchanged identities, four reviewed amendments and one new decision.
Reviewed route digests remain unchanged while protected landing is unresolved.

The appointed manager accepts that bounded authority review and releases the
shared nonvisual editor/popup models, host service, read-only protection
accessors and their focused tests to the existing dev-agent. Actual visual
translation remains pending the Claude target and rendered independent review.
This is implementation permission within the owner's request, not a source or
runtime PASS. Existing arboard clipboard transport is X11/XWayland; reuse must
report provider availability truthfully and cannot claim native Wayland
selection transport or enable a new dependency without its required authority.

### Shared-actions visual review and implementation release

<!-- DISPOSITION:NATIVE-PROJECT-DOORWAY:NPD-C09-FILE-ACTIONS-VISUAL -->

The actual Claude-owned target is
`a190a241194e401a7882ba3b29fa6b9f047996dfa38d6df4f9ded343016ee08a`.
Independent `/root/closeout_review` returned
`BOUNDED_PROTOTYPE_AND_AUTHORITY_PASS` with no remaining target findings,
including PM063 at
`7770e85d80969868296bfd3812b2da5761648790813984675e34633bdc8e6fd8`.
Its latest capture/review receipt is
`7c38aadd5b884926f89315fa1b9303add0b5aa7907f1ecb8ef17880b3e054f0a`:
five fresh busy/table images plus18 byte-qualified unchanged dialog images.
All CSS and26 other dialog fragments are unchanged. The earlier corrected
`e3545dd8` target and review receipt `81f62842` remain historical, with initial
wording findings and the source-pin mismatch noncapture preserved.

The review confirmed a concrete internal race: navigation into a folder being
renamed could start genesis after the worker's last epoch check. The bounded
correction makes Create/Open unavailable until the shared host action and its
receipt resolve. Reverse admission retains protection until actual Open-worker
retirement, including after Cancel/reopen. Navigation, editing and Cancel remain
available; nothing runs automatically. Deterministic implementation controls
and ordinary native review still need to establish this behavior.

The appointed manager accepts this exact bounded visual/interaction target and
releases full native translation, integration, test/fix and a fresh pinned QA
candidate to the existing dev-agent. Both New/Open must use the shared
popup/editor/service. This is no native, clipboard, AT, runtime, HUMAN,
production or landing PASS. Actual source/runtime/rendered independent review
is still required; b015 and the protected landing boundary remain unchanged.

### Shared-actions nonvisual checkpoint review

Independent `/root/closeout_review` verified the 14 archived source pins at
checkpoint `d17fa6bd56d6a6b6df9a7ab54eef3fd78b5b0280c1ca727ae47f1c9857b78c2d`,
source map `f0315264577fa087712318a649ac2426a33ef5ffb37bb2a6bab7e73b32c694fc`.
Actual guarded logs contain 23 passing executions; four failed attempts remain
retained. Source health passed for 3,217 files. These receipts cover the archived
shared editor/popup, filesystem service and path/protection helpers, excluding
subsequent integration changes.

The review requires correction **FA-NV1**: Project classification must protect
declared `settings.checking`/`settings.output` and native `.ids` backing paths.
The existing implementer is correcting the classifier and adding refusal and
no-write controls within the retained grant. Ordinary unrelated host paths must
remain usable. This checkpoint is not accepted while that finding remains open;
controller concurrency, native workflows and clipboard delivery also remain
unverified. No landing or full-qualification claim follows from these tests.

The subsequent bounded correction passed independent source review at
`safety.rs` `c8137b70f81bcc53d544ffa86b669009754b990df007dd608bb7c8d5a0d2f4be`
and `tests.rs` `ae49230242507f0274df1b3d5af401b598eeb6211803630414dfa4706c6459fd`.
FA-NV1 is resolved in that source; focused execution and integration review
remain pending. This does not relabel the original checkpoint as accepted.

The subsequent independent checkpoint-2 review verified all 74 archived Rust
pins in map `9fc1ae5757f27c9239b4c3e9e6976d70df78e2a672fe3fbe4591882141d5d623`
and the actual 11 passing host-service tests, including FA-NV1 and the read-only
availability probe. Shared ownership and both admission guards passed source
review. Two corrections remain for the next delta: **FA-INT1**, visible and
accessible waiting explanations for pending host work, including reopened
dialogs; **FA-INT2**, binding browser pointer actions to the parent/list/entry
actually presented, so an unpresented scan cannot retarget an old row index.
The first controller run forced X11 and is retained only as a historical
diagnostic; it is excluded from required Wayland delivery proof. Corrected
Wayland controls, native workflows, rendered proof and final delta review
remain open. The existing implementation grant covers these corrections.

### Shared-actions final source/runtime/rendered review

<!-- DISPOSITION:NATIVE-PROJECT-DOORWAY:NPD-C09-FILE-ACTIONS-QA -->

Actual independent `/root/closeout_review` returned
**BOUNDED_SOURCE_RUNTIME_RENDERED_QA_PASS** for the 80-file source map
`f1336e25d96bd7bbedc62437d2b5a05d3040e23604c062c664ceb6a94fead734`
and GUI `ba7159dbf9f0286b9fbef972b12feb8757ef7a17976b597eb2a41ce56dcfea8f`.
All archived/current pins and independent GUI/CLI/daemon copies match; the
actual guarded build completed with sources unchanged. The manager accepts
this bounded review and the separately verified final QA package below;
protected landing, task closure and full qualification remain open.

Both New/Open actually use the shared popup, single-line editor, browser
controller and asynchronous typed host service. New Folder/Rename stay inline;
ordinary field selection and clipboard editing work. The host service refuses
replacement and protected Project/storage/session relocation. Completed explicit
host actions persist after outer Cancel; cancelled naming drafts write nothing.

FA-NV1 and FA-INT1..4 are resolved: backing-path protection, truthful waiting
status, presented-entry pointer identity, release ownership and long-path paint.
The last correction projects only visible UTF-8 text through the existing cache;
complete value, caret, selection, pointer mapping and global bounds remain intact.
Its actual-cache negative reproduces the old two-row wrap; the corrected test and
eight affected-form tests pass. The first attempt was a resource refusal, not a
failed test. The extracted local-path guard also passes 15 guard tests and
its live check, with independent review; the original guard failure is retained.

The final GUI has 20 targeted default/620 Home/End/selection captures and 40
corrected minimum captures. The reviewer directly inspected all20 targeted and
26 relevant minimum frames. Exact unchanged owners qualify the earlier
`33462ad6` default64 observations: New/Open file actions, collision/retry,
clipboard/provider failure, eligible-parent Project/storage protection, GUI
Create/Open, eight Units and original shell/child/environment preservation.
That cohort's long-Location paint is superseded, not assigned a blanket visual
PASS. Earlier source/control and prototype receipts retain their exact scope.

Only the owned Project test copy was moved out of Cargo output, atomically into
existing ignored task scratch, preserving all8 byte/mode entries. The receipt is
`0e493a432dbac80eace6e3b1e8b7d8afd3030768975aad9cf31c1128c0dd2437`.
Executed paths remain historical; no other QA target, external workspace or owner
data was moved. Future copied classification fixtures use owned temporary storage.

Datum is native Wayland/WGPU/Vulkan; its existing clipboard bridge is XWayland.
This is no KWin-workflow, native-selection, HUMAN, full AT/platform or ordinary
final window-Close claim. Inherited filename glyph coverage is separate intake
`dat-ui-filename-glyph-coverage-o53p`. Existing convergence failures remain
`dat-wq2s`; relevant checker/assignments are unchanged from HEAD. The actual
visual-owner capability gate remains unresolved; reviewed route digests are not
refreshed to bypass it. No source landing, task closure or S5A/successor release.

Final package review returned **BOUNDED_QA_PACKAGE_AND_DELIVERED_LAUNCHER_PASS**.
The reviewer checked 76 package/runtime bindings, all 80 archived-source entries,
the selected logs and the actual launcher screenshot/diagnostic. The manager
independently verified all 1,080 catalogue hashes and the selected logs' 63 passing
executions / 59 distinct cases; the two broadly filtered native cases were also
executed separately. The final GUI has 61 fresh captures including the actual
launcher smoke. The earlier 64-frame default workflow remains qualified reuse.
The launcher starts from Home with a running unbound shell on native
Wayland/Vulkan; isolated XDG, owned Weston and the existing XWayland clipboard
provider remain explicit limits. Mutable launcher telemetry is excluded from the
frozen catalogue; its diagnostic copy is retained with the smoke receipt.

The compact handoff is
`.datum/native-project-startup-repair/c09-file-actions-final-handoff-summary.json`
(`52e02f5fb65324170f10f46dd1e36724420175b0e5878a68e7df8da736fd2a23`),
with catalogue `dace555bc6dd9391271119cf743f9a2c36a0e38952dbe8838be093807c9116c4`.
The manager's actual bounded acceptance is
`target/qa/file-actions-f1336e25/manager-review.json`
(`8da9166f269036020c81f264c3a8dfc4df4e1ca2c2edc401ba55010c52831065`).
These ignored artifacts add no tracked evidence directory or external workspace.
Launch the ordinary application from an active Wayland session with:

```sh
/home/bfadmin/Documents/datum-eda/target/qa/file-actions-f1336e25/launch.sh
```

The QA package is available. Source landing remains blocked by the actual
visual-owner capability gate. Its remedy is a properly authorized visual-owner
session completing the normal commit, followed by reviewed route reconciliation
and the remaining source/state landing. This acceptance does not close NPD-C09,
claim a source commit or release any broader qualification/successor boundary.

### Assignment and evidence boundary

The synchronized NPD-C09 claim is the umbrella for manager-owned task/state/spec
records, dev-agent native source/tests, and actual Claude visual-source work.
The dev-agent thread is `01a14799-fef3-7e01-b079-2fcae52aa45d`; the manager remains
`01a13df1-7a0d-72c1-bbdd-f0d96d481598`. Claude alone may reconcile the existing
`start-page-study.html` New form/keyboard order and bounded Open/browser states.
Codex will not edit that prototype. The retained claim was explicitly reconciled
and renewed for the owner-approved refinement, without transfer or another
session's edits. The earlier 37 source files were preserved and verified in
ignored task scratch before editing. Source implementation follows the actual
owning-lane target and independent review; no review PASS or prototype edit is
inferred from delegation. The existing visual-owner landing blocker remains
unresolved and is not waived by this refinement.

Independent closeout review verified 145 unique unchanged members across the
foundation (4), rollout (28), workspace (80) and editor-shell (35) routes against
its complete previously reviewed corpus. The two rollout C08 closure records
match its accepted final pins; all other members are byte-identical. Reuse is
limited to that corpus. Review every C09 changed member and reconcile affected
route digests in this change; no new report tree is created.
Raw C09 captures, logs and task scripts stay in the existing ignored
`.datum/native-project-startup-repair/c09-*` scratch; compiled outputs stay under
`target`. No external workspace or owner Project directory is a required input.

### Historical Browse visual target and manager disposition

<!-- DISPOSITION:NATIVE-PROJECT-DOORWAY:NPD-C09-VISUAL -->

Before the single-dialog refinement, the appointed manager adopted the C09
New/Open/browser target at prototype
SHA-256 `b00a2fbdb32ff228fb6f118df5f094f349e2a35ee0527ac3cc37baccb58ce1f2`.
The actual owning-lane session `2aaa642b-8e49-46cf-b7bf-b2f682810d2f` made the
bounded amendment and corrections; its successful edit receipts report no
permission denials. Independent closeout review returned
`BOUNDED_PROTOTYPE_AND_ROUTE_PASS` after inspecting the actual DOM/CSS, all 16
final desktop/narrow captures and focused keyboard scrolling of the full-path
preview. The final prototype-only capture receipt SHA-256 is
`03287d36d6fcb95b971fbe092cf2ea80b59ef34a6700ffce062dc418ac81ef83`.
Older captures remain draft attempts, not final evidence. The target retains the
existing Open footer placement, dot-folders, full read-only scrollable path
preview and unchanged Start-page deferral. This disposition authorizes exact
native translation at that checkpoint; it does not adopt the subsequent
single-dialog target or claim native, runtime, production or HUMAN
acceptance. The C09 scenario is a cohesive helper behind the existing
`prove_native_project_startup_gui.py --browse` entry point, not a separate runner.

### Historical integrated dialog target and manager disposition

<!-- DISPOSITION:NATIVE-PROJECT-DOORWAY:NPD-C09-INTEGRATED-VISUAL -->

This target and its receipts are historical for the earlier integrated candidate.
Its Units controls/panel are superseded by NPD-C09-AUTO-UNITS; other retained
behavior supplies the baseline for the current correction.

The manager adopts the actual Claude integrated New/Open target at SHA-256
`8cbe1de0aec971d07298151e2fa12e369ccd83f24c62a6789873b31c08fbe23d`
for native translation. Actual independent `/root/closeout_review` returned
**BOUNDED_PROTOTYPE_AND_AUTHORITY_PASS** for this exact long-refusal delta.
The owning-lane receipt is
`eacddc68c404db7ceb6e05b4aca60286033b47f1cb4ea62f72d3599daffb7029`;
all six new minimum-size captures were inspected, with capture receipt
`9d87c8de5f2084882217572fd23d9c05bf863dbf74904d9aeff296d8d5bcd0dd`.

The preceding integrated target `3ca09f65443b58aa4965057b05755bdd6ab1212ffefe3680b272d94e7d180fe3`
retains its reviewed 27 initial captures across 17 states and six corrected
captures, owning-lane correction receipt
`0bb903f76d1dd10495a9b1347f77360afb503b9cfe15b1a2cec841f26aa3c584`
and capture receipt
`fb54651ac12a49677222f21e5b4162f3b0907c6e0ecc3fb3d8349e4db8921302`.
The four governing members and all 145 route members were reviewed or
source-qualified for reuse. The 17 earlier state-frame lines are unchanged.

The target retains a bounded 760-by-720 default and 620-by-620 minimum frame,
fixed header, refusal band and footer, and scrolling body. New has name/optional
folder rows above inline navigation and Units/details columns; Open uses one
navigation column. A refusal's retry action remains in the fixed band. A long
reason has a visible cue naming the existing destination preview. That preview
keeps the exact full path first, followed by a separate **Full refusal details**
heading and the complete reason. These details never become path authority.
Keyboard and body scrolling expose the full text and all eight Units. Derived
folder fields skip Tab order; Units groups have valid explicit accessible labels.
The existing Retry/Open Preferences references retain prior target states and
add no new action or scope. These are prototype and authority findings only:
native input, source/runtime behavior, full assistive-technology qualification,
HUMAN acceptance and landing remain open. The protected visual-owner commit
blocker is retained without bypass.

### Automatic Units target and manager disposition

<!-- DISPOSITION:NATIVE-PROJECT-DOORWAY:NPD-C09-AUTO-UNITS-VISUAL -->

Actual independent `/root/closeout_review` returned
**BOUNDED_PROTOTYPE_AND_AUTHORITY_PASS** for the automatic-Units target at
`f3adca6ba5c2d366e02d43cd3f1c9dc49b7f38b2b648047b5c9e1984946d95b2`.
The manager adopts that exact target for native translation under the owner's
recorded direction. The review inspected DOM/CSS, all 22 initial frames across
13 New states and all four corrected G-E/G-F frames. The copy correction changes
G-E/G-F/G-G among the retained target lines; CSS and five Open frame/law lines
are unchanged. Default naming, full-width navigation and full destination fit
without body scrolling; minimum/error states retain usable scrolling and fixed
error/reread/footer controls. Error guidance claims no shipped GUI store-repair
or migration action. Units configuration remains outside New.

The actual owning-lane correction receipt is
`272f8e9bdf9cc664fcf5b61fa04e57b009eeeac57f8e7563cf4da02dd5fa493a`.
Initial capture receipt:
`3462b5ad11ad96025f0a137685b650f346665aa5cd886a5cde0e69e3298251c9`;
corrected capture receipt:
`0068bb163b6e9ecf79b430642341d089b1b5ff46abb5dfe008759de9487cf90e`.
The four retained route memberships remain 145 unique members/147 occurrences;
138 unchanged members reuse adequate reviewed evidence, and seven changed
members were actually reviewed. No new full-corpus reread, native/runtime,
HUMAN/full-AT, landing or production acceptance is asserted. The a42d6eb5 QA
archive and all pre-C09 receipts remain preserved at their original scope.

### Required delivery

- Deliver one complete New dialog and one complete Open dialog with inline
  navigation, coherent keyboard/accessibility order and truthful action states.
- Prove ordinary native Wayland New -> enter name once -> navigate parent ->
  inspect destination -> Create with automatic Global Units, and Open -> select Project -> Open,
  without a separate selection page. Verify the optional folder override and
  return to the default, eight-item Units receipt, engine admission and original
  terminal survival.
- Prove cancellation, correction/retry, unreadable/invalid locations, occupied
  destination, stale scan completion, long names/scrolling and keyboard selection
  with no unintended filesystem or Project mutation. Preserve C07/C08 safeguards;
  use owned scratch, never the owner's project directories.
- Run focused meaningful regressions, required source/dependency/governance gates,
  inspect rendered states, obtain actual independent review and manager acceptance,
  land the source, and provide a pinned Wayland QA cohort. Existing unrelated
  Clippy/accessibility/full-qualification limits remain explicit and separate.

<!-- EVIDENCE:NATIVE-PROJECT-DOORWAY:NPD-C09 -->

At this historical visible-Browse/direct-Location checkpoint, the candidate had
actual independent bounded source/runtime/rendered QA PASS and appointed-manager
QA acceptance. The earlier automatic-Units, integrated and Browse candidates
remained historical. Landing, task closure and production acceptance were then
open because the protected visual-owner commit was unresolved. The final C09
acceptance above records its later authorized resolution.

<!-- CHECKPOINT:NATIVE-PROJECT-DOORWAY:NPD-C09-BROWSE-AFFORDANCE-QA -->

### Visible Browse and direct Location QA candidate

Actual independent `/root/closeout_review` returned
**BOUNDED_SOURCE_RUNTIME_RENDERED_QA_PASS**, with no mandatory findings. The
appointed manager accepts this bounded QA candidate under the recorded owner
direction. Its 48-file source manifest is
`b01589871fd2153fff54eeedc79e5bef8803a1ea3f610fbb4fcac206b9989f80`:
46 Rust files and two proof helpers, based on `9857969199fc16545d843ecb516bd6ad5a15fe2f`.
The independent source and executable copy is
`target/qa/browse-affordance-b0158987`; GUI SHA-256 is
`a83d500e53109afe8ccd899c899e45141216868e167e625039d6ac49ea2661a9`.
Its separate `manager-review.json` records actual acceptance without rewriting
the frozen implementer handoff or build metadata.

New and Open start with Location and visible Browse. Browse reveals and focuses
navigation in the same dialog; hidden navigation has no pointer, keyboard or
accessibility stops. A typed Location can go directly to Create/Open. New uses
the derived folder or explicit override; Open uses a valid selected child from
the current listing, otherwise the current typed Location. Discovery failure
does not override canonical final-action validation. Location edits invalidate
stale selections and restart expanded discovery. Entering a folder from a
`~/…` or relative Location resolves through the existing GUI path resolver.
New keeps automatic Global Units with the existing error-only explicit reread.

Review verified all 104 evidence bindings and independent executable/dependency
copies. The accepted focused evidence is 69 passing executions, 55 distinct
cases and two existing ignored executions; this is not a full-suite result.
The corrected ordinary Wayland run passed with 38 inspected frames, including
pointer/keyboard Browse, direct typed Create/Open, selected-child Open, actual
tilde/relative folder entry, discovery-permission refusal, automatic eight-item
Global `human_gui` receipt, stable Project bytes and original shell/child/environment.
All owned PTYs exited before the final GUI SIGTERM.

The 24 inspected minimum-size frames retain their actual pre-BR3 source
`547d406f` and GUI `ef3c5396`. Independent review verified the exact production
delta and identical minimum helper AST, renderer and protocol: those scenarios
never enter the corrected selected-row branch. The 19 New and 22 Open test
executions have the same explicit unaffected-source qualification; 28 navigation
executions and the corrected default run use the final Rust. The failed first
BR3 runtime attempt remains preserved: its helper assumed the wrong GUI cwd.
The corrected helper reads the actual owned process cwd and records the exact
entered path; the fresh second attempt passes. No failed receipt was relabeled.

Launch the reviewed QA copy from the owner's Wayland desktop:

```sh
cd ~
env -u DISPLAY -u WINIT_UNIX_BACKEND \
  /home/bfadmin/Documents/datum-eda/target/qa/browse-affordance-b0158987/debug/datum-gui
```

Datum used native Wayland/WGPU/Vulkan in owned nested Weston; X11 transported
input to Weston only. No new KWin, HUMAN, full assistive-technology, ordinary
final window Close, installed-agent, Preferences-setter or full qualification
claim is made. Existing Clippy/accessibility/terminal/text-editing intake remains
open, including Ctrl+A/clipboard limitations. Earlier QA copies stay frozen.
Protected visual-owner landing and reviewed route reconciliation remain blocked;
no source commit, task closure, production acceptance or S5A/successor release.

<!-- CHECKPOINT:NATIVE-PROJECT-DOORWAY:NPD-C09-AUTO-UNITS-QA -->

### Historical automatic Units QA candidate; superseded by visible Browse QA

Actual independent `/root/closeout_review` returned
**BOUNDED_SOURCE_RUNTIME_RENDERED_QA_PASS**, with no mandatory findings. The
manager accepts this bounded QA candidate under the recorded owner direction.
The frozen source catalogue is
`6183c81e505987692c6a7527c150e1501363757460494f087c8b3af94e7d5e1e`:
45 Rust files and two proof helpers, based on
`9857969199fc16545d843ecb516bd6ad5a15fe2f`. Independent source and executable
copies are in `target/qa/auto-global-6183c81e`. The separate `manager-review.json`
records acceptance without rewriting the frozen implementer handoff.

New contains naming, the derived folder or optional override, inline parent
navigation, complete destination and Cancel/Create. Opening it validates and
pins Global defaults automatically; Units controls, summary, source metadata
and their focus/accessibility stops are removed. Ordinary native Create produced
the unchanged eight-item Global `human_gui` receipt. The existing Project
Preferences doorway exposed its eight Units controls; opening/cancelling it
preserved all five fresh-Project files. This does not claim a new setter proof.
The original terminal shell, child and environment survived creation.

Actual focused tests passed 40 executions across 33 distinct named cases, with
two existing ignored GPU cases. The guarded matching build passed. Source
health passed for 3,206 files; unchanged dependency/resource inputs reuse their
adequate passing evidence. Independent review inspected all 24 default and
14 minimum native captures. At the actual 620-by-620 minimum, fixed refusal,
Create and error-only reread remain reachable while body and full preview
scroll. Explicit reread after source repair restores eligibility without
creating. No silent Factory fallback or new GUI repair mechanism is introduced.

Default runtime evidence retains its actual `5abd2500` helper source. Later
helper changes affect only the unexecuted minimum branch; verified unchanged
default paths justify reuse. The first minimum attempt remains failed/incomplete:
its helper incorrectly demanded another frame after the correct refusal had
already been presented. The minimal helper correction retains current-frame,
focus and refusal checks; the second minimum attempt passed. Rust and binaries
were unchanged. The 97-entry evidence catalogue and paired JSON bind both
successful runs, the failed attempt, exact source and executable copies.

Limits remain explicit: native Wayland/WGPU/Vulkan under owned nested Weston
does not establish new KWin, HUMAN, full assistive-technology or platform
qualification. Default proof exits owned PTYs before GUI SIGTERM; minimum proof
cancels New before SIGTERM. Neither establishes ordinary final window Close.
Fresh-Project proof has no authored journal; unchanged Open evidence is reused
at its prior scope. Existing unrelated Clippy and other intake remain open.
The a42d6eb5 candidate stays frozen. Protected visual-owner landing, reviewed
route reconciliation and source/factual closure remain pending; no digest
refresh, gate bypass, production acceptance, S5A release or successor grant.

Launch this reviewed candidate on the owner's Wayland desktop:

```bash
cd ~
env -u DISPLAY -u WINIT_UNIX_BACKEND /home/bfadmin/Documents/datum-eda/target/qa/auto-global-6183c81e/debug/datum-gui
```


<!-- CHECKPOINT:NATIVE-PROJECT-DOORWAY:NPD-C09-INTEGRATED-QA -->

Historical source-pinned QA: the Units presentation is superseded by the reviewed
automatic-Units correction. These observations remain valid only at a42d6eb5.
### One-dialog QA candidate; protected visual landing remains open

Actual independent `/root/closeout_review` returned
**BOUNDED_SOURCE_RUNTIME_RENDERED_QA_PASS** with no remaining mandatory finding.
The manager accepts that bounded QA receipt. The frozen source catalogue is
`a42d6eb513188f82a862cc184fc7f903b2b644852687e620fa3a703ad26ba708`:
45 Rust owners and two proof helpers, based on `9857969199fc16545d843ecb516bd6ad5a15fe2f`.
The source is uncommitted. Independent source/executable copies are available in
`target/qa/inline-a42d6eb5`; its later `manager-review.json` records this acceptance
without rewriting the frozen implementer handoff metadata.

New now combines name, visible derived folder/optional override, parent
navigation, full destination and eight Units in one form. Open directly uses
the highlighted or current directory. Ordinary native evidence verifies
Create, both Open paths, correction/refusal/Cancel, an eight-item Global
`human_gui` receipt, five unchanged fresh-Project files and original shell,
child and environment preservation. At the actual 620-by-620 client minimum,
fixed alerts, final actions and Global reread remain usable while the body and
full destination/reason preview scroll independently. The accessibility preview
carries the full reason separately from its exact path value.

The five actual focused logs contain 63 passing executions across 50 distinct
named cases and two ignored GPU cases. The corrected preview-binding assertion
was rerun after its final source edit. Source health passes for 3,207 files;
dependency/resource evidence is reused only for unchanged inputs. Project-state,
spec-governance and spec-parity checks pass (101 Frontier items, 570 classified
specifications and 30 inventories). Evidence traceability reports the three
affected routes as stale; its manifest remains unchanged while protected
visual-owner landing is unresolved. No digest refresh bypasses that gate.
The matching build and both final native runs pass their executed assertions. All 27 normal
and 18 minimum captures were independently inspected. An early loading frame
is explicitly excluded from refusal proof; its corrected later frame is used.

The frozen implementer handoff is
`25d81d978c4dfafacb8acf7a65949f6f8d48bcdb8d3cd002d3808c7e28647243`;
its 226-entry evidence catalogue is
`0a18ac5f9f0b6c996b5866427969864606734d842adea73a6598c1dc77ee153b`.
The paired JSON records exact binary, native-result, command and manager-receipt
bindings. Failed compile/input/harness/capture attempts remain retained. The
first normal run's residual `/h` input prefix has no proven product cause;
exact replacement checks do not establish input throughput.

Limits remain explicit: nested native Wayland/WGPU/Vulkan is not new KWin,
HUMAN, full assistive-technology or full-platform qualification. The fresh
Project has no authored journal. The normal run closes its PTYs before final
GUI SIGTERM; the minimum run only cancels forms before SIGTERM. Neither proves
ordinary final window Close. Existing Clippy/accessibility/terminal intake stays
open; conventional field-selection/replacement is separate intake
`dat-project-dialog-text-editing-y9ja`. Protected visual-owner landing still
requires the authorized owning session, then reviewed route reconciliation and
source landing. No gate bypass, closure, production acceptance or S5A release.

Launch the reviewed candidate on the owner's Wayland desktop:

```bash
cd ~
env -u DISPLAY -u WINIT_UNIX_BACKEND /home/bfadmin/Documents/datum-eda/target/qa/inline-a42d6eb5/debug/datum-gui
```

<!-- CHECKPOINT:NATIVE-PROJECT-DOORWAY:NPD-C09-QA -->
### Historical C09 Browse QA candidate; visual-owner landing remains blocked

The frozen working-tree candidate is based on `9857969199fc16545d843ecb516bd6ad5a15fe2f`.
The 37-file source catalogue (35 Rust files and two proof scripts) is
`e2d4605652c2602c1b0dc279f53e259a5ec1d53f2bc2fa059179d18d801dd97c`.
Actual independent `/root/closeout_review` disposition is
**BOUNDED_SOURCE_RUNTIME_RENDERED_QA_PASS**, with no remaining blocking
Browse-specific implementation finding. The manager accepts that bounded QA
receipt only; NPD-C09 landing, task closure and production acceptance remain open.

New and Open share read-only native folder browsing. Selection fills the full
original form path; New uses an explicit new-folder leaf. Cancel preserves the
draft, eight Units and terminal state. The reviewed corrections preserve exact
whitespace/Unicode path identity through actual Create/Open adapters and retain
Main terminal redraw when a browser scan completes simultaneously. Long paths
wrap in a keyboard-scrollable preview, with visible-list accessibility updates.

Actual final guarded tests pass 13/13; the matching build passes. Retained
source-qualified cohorts contain 87 passing executions / 81 distinct test pairs,
three distinct ignored tests and one unrelated accessibility wording failure
(`dat-selection-a11y-pending-copy-ak7m`). This is not a full-suite PASS.
Source health (3,203 files), dependency, resource and affected structural checks
pass. Full drift stops at the same 16 errors in eight unchanged engine files
(`dat-lk0l`); later full gates were not reached.

Final ordinary native evidence is `c09-wayland-attempt-5`: 17 independently
inspected frames, pointer/keyboard Browse, missing/empty/occupied refusal,
New/Open cancellation, a 1,258-character Home/PageDown/End/Home path preview,
wheel/accessibility coherence, GUI Create/Open and the eight-item Units receipt.
Original shell/child identity and environment survive; five catalogued Project
files remain unchanged through Browse/Open. This fresh Project has no
journal-bearing history, so this run is not new existing-history proof.

Datum itself runs on Wayland/WGPU/Vulkan. Owned nested Weston and its outer
input transport do not establish KWin input, HUMAN, latency or full-platform
qualification. Final SIGTERM is not ordinary window Close. Main right-edge
chrome is partly outside the capture; no whole-Main visual PASS is inferred.
The retained rapid-typing refusal (`dat-terminal-input-busy-loss-2oin`) is not
repaired here; slower proof typing is harness pacing, not a throughput claim.
Failed earlier compile/input/capture attempts remain retained with their limits.

The independent QA copy is `target/qa/browse-e2d46056/debug/datum-gui`, with
matching CLI/daemon, actual dependency records, `source-manifest.json` and
`build-info.json`. All copied hashes match the tested binaries and use separate
regular files. GUI and CLI help probes pass. Start it from Home with:

```bash
cd ~
env -u DISPLAY -u WINIT_UNIX_BACKEND /home/bfadmin/Documents/datum-eda/target/qa/browse-e2d46056/debug/datum-gui
```

The actual visual-owner session authored and reviewed the prototype, then tried
the normal scoped commit. The repository gate refused because that session lacks
its required launcher capability. HEAD remains `98579691`; the index is empty.
No protected edit, marker bypass, authority refresh or landed/closure claim is
performed to absorb that refusal. The manager's provisional manifest refresh was
restored to the original bytes; three reviewed routes remain intentionally stale
until the authorized visual-owner commit and same-change reconciliation.

The exact remedy is an authorized visual-owner session landing its reviewed
`docs/gui/prototypes/start-page-study.html` (SHA-256 `b00a2fbd…`) through the
normal gate. The manager can then reconcile the already reviewed routes, land
the frozen implementation and finish factual closure. No new product decision
or routine owner approval of Browse behavior is needed. Existing shared-tree
ownership is retained explicitly pending this handoff; S5A and successors remain
held. Scratch/logs/captures stay ignored inside the existing task directory.
