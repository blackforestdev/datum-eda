# Manual capability baseline: WDQ-C01

Date: 2026-09-05
Status: partial audit evidence; no workflow or product acceptance
Owner: WORKFLOW-DELIVERY-QUALITY / dat-workflow-delivery-quality-xgj

## Evidence boundary

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

## Limits and remaining WDQ-C01 work

This is a committed first audit unit, not WDQ-C01 completion evidence. Before
closing the step, obtain a verified current build identity and reproducible
fixture provenance; exercise reachable production input paths in a controlled
session; record observed failure/cancel/reopen behavior; and finalize the gap
ownership mapping. Unavailable paths stay unavailable rather than being filled
with engine-only tests. Do not proceed to dependent research or ratification by
pretending these missing observations passed.

The finding already supports one concrete gate requirement for later design:
consumer activation must be checked against actual dispatch and behavior, not
the presence of an action string. It does not establish a complete gate design.
