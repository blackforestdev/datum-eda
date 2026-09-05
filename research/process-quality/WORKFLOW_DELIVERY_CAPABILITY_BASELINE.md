# Manual capability baseline: WDQ-C01

Date: 2026-09-05
Status: bounded WDQ-C01 baseline complete; no workflow or product acceptance
Owner: WORKFLOW-DELIVERY-QUALITY / dat-workflow-delivery-quality-xgj

## Evidence boundary

The original observations below are preserved as historical evidence. The fresh
build and isolated-input continuation at the end supersedes their unverified
build/fixture/input limitations for the specific paths exercised. It does not
certify unavailable editing workflows or all platform variants.

Current source inspected at `ad7b08461fa45aa80fca3543116639fa8682933e`.
Preferences/Units decisions, plans, research and three protected HTML prototypes
were concurrently dirty. They were not edited or treated as reviewed authority.
This report describes implementation observations; it does not amend product
intent or reassess ratified specifications. Existing Frontier tasks remain owners
of their implementation scope.

The existing `target/debug/datum-gui` binary was used for diagnostic launches.
Its build commit is **unverified**. Its mtime is 2026-09-04 13:42:04 UTC, size
590799072 bytes, and SHA-256 is
`ccf69cef456d971910c6094316bfb3b89eae2065fbc652f63818c66c07593245`.
No fresh build or Rust test run occurred. Source findings and binary observations
are deliberately recorded separately; this executable cannot certify current
HEAD. The rendered `rev da3786` label is not established as a build identity.

## Fixture and environment

An existing native converter workspace was copied from
`/tmp/datum-eda/gui-imports/datum-test-134cba054497b2ce` into
`/tmp/datum-wdq-c01-eqfdWO/project`. Its name is `datum-test Datum Workspace`,
project UUID `b6702e62-e0f7-4a26-b33e-64b407a7ded6`. The source board exists at
`/home/bfadmin/Documents/kicad_projects/Datum-eda/datum-test/datum-test.kicad_pcb`,
but the converter receipt and exact source-to-workspace correspondence were not
audited. This is an existing project display fixture, not native manual-authoring
acceptance evidence. No design was fabricated or reimported for this audit.

The copied project and board digests were identical before and after the
offscreen capture:

| Input | SHA-256 |
| --- | --- |
| project.json | `456f2d00f51b06c7df858d30ccc8ed1642455c930cc842792cf10b705b428fc3` |
| board/board.json | `cbe341184ce5f34fd8db597e32dc984f52c6cca69313b7086a92e7c40ace1916` |

These two checks do not establish whole-project immutability. The scratch copy
isolates possible startup writes from the source project. Captures used
`XDG_CONFIG_HOME=/tmp/datum-wdq-c01-eqfdWO/config` and separate `DATUM_GUI_LOG`
paths; they did not open or change the active sessions' Preferences windows.

## Observations and reproductions

### Ordinary startup

Command: `target/debug/datum-gui` with no project argument.

Observed: exit 1, `datum-gui error: launch state load failed: resolve GUI launch
review context`. No project chooser was demonstrated. The executable's help
identifies itself as a Phase 1 read-only board GUI.

Current source independently supports the missing entry path:
`crates/gui-app/src/app_bootstrap.rs`, `GuiArgs::resolve_request`, requires
`--project-root`, `--board`, `--schematic` or `--demo-known-good` unless an artifact
is supplied. This is source evidence of the launch contract, not a claim about
all possible packaged launchers.

### Existing project display

The following command exited 0 and produced the inspected artifact:

```sh
env XDG_CONFIG_HOME=/tmp/datum-wdq-c01-eqfdWO/config \
  DATUM_GUI_LOG=/tmp/datum-wdq-c01-eqfdWO/render.log \
  target/debug/datum-gui \
  --project-root /tmp/datum-wdq-c01-eqfdWO/project \
  --visual-test --exit-after-screenshot --open-menu File \
  --window-size 1280x768 \
  --screenshot-out /tmp/datum-wdq-c01-eqfdWO/file-menu.png
```

Committed artifact: [File menu capture](evidence/wdq-c01-existing-binary-file-menu.png).
Artifact SHA-256:
`2f64939fd132f5c88427db22636b59a5b8160398e495d53a697a781701134834`.
It shows board geometry, grey File actions, and `Schematic (coming)` in the
second pane. The menu was seeded open by a capture flag. This is render evidence,
not mouse/keyboard interaction proof or cross-renderer prototype acceptance.

A separate ordinary window launch used the same scratch/config paths with
`WAYLAND_DISPLAY` unset and a 120-second timeout. The application configured a
1333x800 Bgra8UnormSrgb/Fifo surface, reported `window visible`, then reported
`close requested` about 31 seconds later and exited 0. No successful targeted
input was sent before closure. The owner subsequently explained that the temporary
X11 window stayed above other windows and obstructed the desktop, so they closed
it. This is an audit-environment disruption, not an application crash. Future
interaction testing must use an isolated virtual display, not the owner's desktop.
Treat interactive behavior as unverified; do not
infer successful tool, menu or persistence behavior from a visible window.

### Current-source action boundaries

`crates/gui-app/src/main.rs` rejects non-Select workspace tools in
`set_workspace_tool`; `active_tool_is_authoring` returns false. Existing gesture
handlers therefore do not establish reachable manual authoring.

`crates/gui-protocol/src/gui_menu_model.rs` enables only GUI-local or submenu
bindings (subject to Project presence), not public-verb bindings.
`crates/gui-app/src/runtime_menu_actions.rs` dispatches GUI-local bindings and
reports refusal for the remaining actions. These are direct implementation
boundaries; a menu entry naming a public verb does not establish a GUI write path.

There is also a narrower readiness defect: any GUI-local binding is eligible
for enablement, even when the dispatch implementation lacks its action. Examples
`view.layers`, `window.documents` and `help.about` occur in the compiled-in
`docs/gui/menu_model.json` data but have no matching runtime handler. In
`runtime_view_actions.rs`, they fall through to `View action is unavailable`.
Native input reproduction is outstanding. This deterministic source mismatch
is captured as intake bead `dat-gui-local-readiness-uev`, not repaired by this
audit and not used to authorize the missing features.

## Workflow coverage and gap ownership

These are observations of bounded paths, not global assessments of each engine.
"Unavailable" below means the inspected native entry/action path is unavailable;
other paths remain unverified unless explicitly exercised.

| Manual scenario | Evidence/classification | Existing scheduled owner |
| --- | --- | --- |
| Start, create a Project and reopen it through the GUI | Ordinary startup fails in the binary; current File bindings do not supply New/Open/Save operations. Argument-based existing-project display is partial. Creation/reopen workflow unavailable on these paths. | GUI-WRITE-PATH (`dat-gui-write-path-qiu`); startup doorway ownership must be reconciled during WDQ-C02, not silently assigned |
| Resolve a library part and place it | Engine library and schematic-symbol builders exist; no native placement was demonstrated and workspace authoring activation is disabled. Engine correctness and alternate library entry paths unverified. | GUI-SURFACE-SPECS (`dat-gui-surface-specs-usb`), NATIVE-AUTHORING (`dat-native-authoring-depth-sf9`) |
| Place symbols, wire and inspect connectivity | Native symbol/wire/junction builders exist; tested binary's board workspace shows a schematic placeholder. Manual authoring unavailable through inspected tool path; standalone schematic viewing unverified. | NATIVE-AUTHORING; GUI-SURFACE-SPECS |
| Propagate schematic changes to a board and review them | Forward-annotation proposal builders exist; no connected manual change/review/apply workflow was exercised. Unverified; required upstream authoring absent on inspected path. | NATIVE-AUTHORING and GUI-WRITE-PATH |
| Enter exact board geometry, snap, cancel, undo, save and reopen | Board display observed; non-Select workspace tools disabled in current source. Full edit lifecycle unavailable on inspected tool path. Units correctness, precision and persistence not inferred from a display. | GUI-WRITE-PATH, NATIVE-AUTHORING; selection/inspection owner UVT-S5A-BUILD (`dat-uvt-s5a-build-1wv`) |
| Produce and inspect a manufacturing set manually | Manufacturing builders and menu entries exist, but verb-bound menu dispatch is disabled. Native menu export unavailable; CLI exports and actual output correctness unverified here. | GUI-WRITE-PATH; broader manufacturing acceptance ownership to reconcile in WDQ-C02 |

Engine implementation references inspected for existence include
`crates/engine/src/api/native_write/{genesis,schematic_symbols,schematic_connectivity,forward_annotation,manufacturing}.rs`.
Their existence establishes neither passing tests nor adequate engineering
semantics. No existing accepted feature or historical milestone is reopened by
this table. Preferences/Units acceptance stays with its active owners.

## Original audit limits

This is a committed first audit unit, not WDQ-C01 completion evidence. Before
closing the step, obtain a verified current build identity and reproducible
fixture provenance; exercise reachable production input paths in a controlled
session; record observed failure/cancel/reopen behavior; and finalize the gap
ownership mapping. Unavailable paths stay unavailable rather than being filled
with engine-only tests. Do not proceed to dependent research or ratification by
pretending these missing observations passed.

The initial finding supports one concrete gate requirement for later design:
consumer activation must be checked against actual dispatch and behavior, not
the presence of an action string. It does not establish a complete gate design.

## Fresh-build and isolated-input continuation

<!-- EVIDENCE:WORKFLOW-DELIVERY-QUALITY:WDQ-C01-BASELINE -->

### Build and fixture identity

The following guarded command passed in 2m02s on clean source commit
`6fa5aa3ee4f033385190938b437e0483ab01393e`:

```sh
python3 scripts/run_cargo_guarded.py --workload proof -- \
  cargo build --offline --locked -p datum-gui-app --features visual --bin datum-gui
```

Source paths `crates`, Cargo.toml and Cargo.lock remained unchanged through the
build and audit. Only audit artifacts and audit-owned tracking were changed.
Toolchain: rustc 1.93.1 (01f6ddf75), cargo 1.93.1 (083ac5135); Linux
6.12.101+deb13-amd64. Binary SHA-256:
`b1c7d833e6aefa86e1302cd043ba8ec590a5defa7fbd21f0873a764cad751816`.
Cargo.lock SHA-256:
`0cee76a3005add00e0a63311e937ed7e15d54c0ec57627849b4cb4e756b7387c`.
This is build verification, not a Rust-suite result or production acceptance.

All 74 entries of the existing native fixture's import map name the same source
board and `sha256:08e39410ca4e5c3d417ad049dec45665393f472185c2fc75f451f83f187175af`.
That hash matches the actual source board file. The journal records the native
CLI import transaction (88 operations) followed by a check-evidence transaction.
This establishes the observed fixture's import provenance; it is not a new
import-fidelity audit. The converter is not reimplemented or expanded.

The committed `evidence/wdq-c01-current/native-fixture.tar.gz` contains the
project, board, schematic, rules, journal, import map and check evidence only.
It excludes terminal credentials and session state. SHA-256:
`91198c1cca854debd9780510e4c4ce00048d1e8fc379402aaa8d6ce240087828`.
Extract it into a fresh scratch directory to reproduce the display fixture.

### Isolation and actual input

Weston 14.0.2 ran with `--backend=headless --renderer=pixman --no-config`, a
private XDG_RUNTIME_DIR and private Wayland socket. Its integrated rootless
Xwayland server crashed during window manipulation; this is recorded as a test
environment failure, not a Datum failure. The successful session used a separate
rootful `Xwayland :99 -geometry 1600x1000 -nolisten tcp -ac -glamor off` connected
only to that headless compositor. No audit window was placed on the owner desktop.

Datum ran normally with `--project-root <scratch> --window-size 1280x768`,
isolated XDG_CONFIG_HOME and diagnostic logs. No visual-test, preset menu or
preset selection arguments were used. XTest pointer/keyboard events were sent
through xdotool on display :99; screenshots came from that actual client window.
The successful client was 2097154; the reopened client was 4194306. IDs are
run-specific and must be rediscovered on reproduction.

| Scenario and actual input | Observed result | Evidence under evidence/wdq-c01-current |
| --- | --- | --- |
| Run fresh binary without a project argument | Exit 1, launch context failure; no create/open doorway on that path | no-project.log |
| Open archived native fixture using --project-root | Board displayed at 1280x768 | file-menu.png and input-events.log |
| Click File at (110,17), then New Project at (160,52) | New Project is grey; attempted activation gives refusal, no creation dialog | file-menu.png, new-project-refusal.png |
| Escape, click Move toolbar at (425,49) | Tool remains Select; no authoring gesture begins | move-tool-inactive.png |
| Click board pad at (365,425) | Selected pad highlight; inspector shows ROUNDRECT, L0 and 1.07 x 0.95 mm | pad-selection.png |
| Click Help at (595,17), About Datum at (655,53) | Enabled-looking entry produces View-action refusal | help-menu.png, help-about-refusal.png; dat-gui-local-readiness-uev |
| Click Place at (235,17) | Symbol, Wire and other authoring entries grey | place-menu.png |
| Escape, click Manufacturing at (460,17) | Output-job, Gerber and export entries grey | manufacturing-menu.png |
| Escape, wheel up twice at (365,425) | Menu dismisses; board geometry visibly magnifies | zoomed-board.png |
| Click View at (189,17), Fit at (245,53) | Board fitted again; visible view-fit feedback | fit-restored.png |
| Reopen the same scratch project in a second process | Board displays again; no edit or output fabrication needed | reopened-board.png, reopen.log |

All captures were inspected. Screenshots establish observed states, and the
verbose input log records actual pointer/key events; neither alone claims a
successful authoring lifecycle. `Help > About Datum` now has a native reproduction
of the previously source-only readiness mismatch.

The first window was destroyed using xdotool windowclose; the client remained
running after the Destroyed event and was explicitly terminated with SIGTERM.
This is not treated as successful normal shutdown. The reopened window received
the standard X11 WM_DELETE_WINDOW client message and exited 0 after logging
`close requested`. Both clients and the private display servers were stopped.

`tar --compare` against the committed fixture archive passed after interactions
and after reopen/normal close: every archived file remained identical. New
terminal/runtime sidecars are outside that comparison. No authored change was
possible through the inspected actions, so this demonstrates preservation across
view/refusal/reopen only; save-after-edit, undo correctness and crash recovery of
pending edits remain unavailable/unverified, not passed.

### Completed baseline and bounded disposition

The six workflow rows above now have a verified-build input/display basis for
their inspected GUI boundaries. Selection, zoom, Fit, menu cancellation and
argument-based reopen are observed partial capabilities. Native project creation,
library placement, schematic editing, propagation, exact board editing and
manufacturing are not demonstrated as complete manual workflows. The board
workspace's schematic placeholder does not prove that standalone schematic
viewing is absent. CLI/engine features remain outside this native-input proof.

Startup/session ownership is explicitly captured in
`dat-native-project-startup-vrf` (intake), with GUI-WRITE-PATH as an existing
integration dependency to reconcile in C02. Manufacturing menu reachability maps
to GUI-WRITE-PATH; broader output correctness remains unverified in this audit.
Library/schematic, native authoring and selection gaps retain the existing owners
listed above. The intake records do not reorder or authorize development.

WDQ-C01 is complete as a bounded capability audit: provenance, build identity,
actual reachable input, unavailable paths, persistence limits and gap ownership
are explicit. This closes the audit step only. C02 must research the uncovered
authority/readiness questions before C03 specifies enforcement. No prototype,
Preferences scope, accepted product decision or production status is changed.
