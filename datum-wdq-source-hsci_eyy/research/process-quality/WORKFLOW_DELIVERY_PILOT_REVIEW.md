# WDQ-G01: pilot authority and accessibility review

Date: 2026-09-06 UTC
Scope: WORKFLOW-DELIVERY-GATE-PILOT / dat-workflow-gate-pilot-b3s
Status: source/authority review and candidate contract; not native proof or G01 closure

## Review boundary

The owner requested completion of the authority/accessibility review and pilot
JSON after planning checkpoint `76a89e7c`. This report completes that bounded
review. It does not authorize implementation, claim an independent reviewer,
install trusted configuration, or complete the remaining G01 handoff.

Production observations were checked against the source present at
`e258a68f062899f425b9eb9ca2c74f11247b0053`; subsequent concurrent Preferences edits
are not included in a native acceptance claim. The compiled artifact has not
been rebuilt or exercised in this planning pass.

Read every source and consumer of `workflow-delivery-quality` before authoring
this report and its consumer `specs/workflow_delivery/pilot.contract.json`.
Also read the complete `prototype-editor-shell-and-panes` route: all three HTML
sources (board-editor, schematic-editor, workspace-panes), GUI design and
conformance specs, PM019, PM021, and the Publish visual study brief. Their route
digest at review was
`33e4ef5a297c1fbc52cd93264cbd87e65b53a7153a86277d7f639380ec09f0af`.
None was edited or re-ratified. Menu bindings were read separately; the manifest
does not assign that file or menu_model.json to an evidence route today.

## Authority conclusions

| Authority | Pilot consequence |
| --- | --- |
| Ratified PM041 and adopted gate/adoption contracts | Missing consumers may remain honestly unavailable; every enabled pilot entry needs the real context-eligible handler. PILOT-S01–S05 remain mandatory; no screenshot-only acceptance |
| PM019, Application Shell and GUI Edit Authority | Preserve discoverable View/Window/Help families and eventual product requirements. GUI-local classification is not proof of implemented behavior; no terminal-command substitute |
| PM021, Decision and How it rides the substrate | Focused document/view owns active-editor actions. Fit and menu focus are view/session state, never design mutations. Preserve the recursive tree and other panes' cameras |
| GUI design, Modularity and Design Language | One action identity/result across its actual entries; data-driven menu inventory and existing visual tokens remain controlling |
| GUI conformance, sections 0 and 2.1 | Token values and native geometry checks differ from human composition review. Neither historical mockup pixels nor a current shell golden certify new availability behavior |
| Board/schematic/pane HTML and Publish brief | Preserve one-window shell, pane-local focus, Inspector/Layers binding and future capabilities. These prototypes do not specify executable About/Documents/Layer Visibility dropdown handlers |

No reviewed authority requires a missing pilot handler to appear enabled. No
prototype change is necessary merely to specify truthful availability. Historical
study banners and old schematic-sheet labels are not new owner decisions; the
Publish brief and PM021 explicitly preserve the later one-window/content rules.
Do not copy historical shortcut glyphs or placeholder pixel values into runtime.

The menu-bindings document calls whole families GUI-LOCAL and contains historical
plumbing observations. That classification cannot override PM041 or establish
current functionality. In particular, disabling `view.layers` must **not** disable
working individual layer toggles; disabling `window.documents` must not disable
existing document/pane navigation. New panels, an About dialog, units policy,
authoring and broad menu redesign remain excluded.

## Production path inventory and corrected finding

| Contract surface | Actual production route |
| --- | --- |
| view-fit-menu-pointer | `menu_model.json` View / Fit to Board (`view.fit`) → renderer `MenuItem` hit → `Runtime::activate_menu_item` → `Runtime::activate_gui_local_menu_action` → `Runtime::fit_camera` |
| view-fit-menu-keyboard | Pointer-open View, then existing Up/Down/Enter handling in `runtime_menu_actions.rs::menu_key_intent` / `Runtime::handle_menu_key` → the same menu activation |
| view-fit-shortcut | Editor-owned F/f in `workspace_keyboard.rs::character_action` / `apply` → direct `Runtime::fit_camera`; currently bypasses menu dispatch |
| missing-consumer menu pointer/keyboard | View / Layer Visibility (`view.layers`), Window / Documents (`window.documents`), Help / About Datum (`help.about`); each goes through the same menu activation but has no specific handler in `runtime_view_actions.rs` |

Search of production sources and both menu inventories found no assigned direct
shortcut or marking-menu entry for the three missing consumers. Do not fabricate
one for a test. `view.fit-focused` in visual_runner.rs is seeded Console render
data, not an additional production invocation surface.

**Correction to the preliminary bead/checkpoint:** `HitTarget::FitBoard` still
has a dispatch arm in main.rs, but render_project_filters.rs deliberately discards
its computed button tuple rather than emitting a hit region. The comment assigns
that space to the Revision Navigator. This is an inactive legacy path, not a live
side-panel button. The contract does not demand its reintroduction. Relevant-input
coverage includes both files so future resurrection cannot escape review.

`GuiMenuItem::is_enabled` currently checks binding class and Project presence;
menu_chrome.rs uses that result for paint while emitting hit regions even for
disabled rows. The generic disabled branch currently says an open Project is
required, which would be false for an absent consumer. Disabled focus inspection
must remain possible, but Enter/click must explain rather than execute.

`Runtime::fit_camera` resolves the focused viewport, returns without effect if
that resolution fails, and otherwise calls `CameraState::fit_to_bounds` in
gui-viewport/src/camera.rs. Its comment mentions a fallback to the board that the
actual early-return code does not perform. The contract follows focused-pane
authority, not that stale comment: unresolved/non-camera content is unavailable,
not a successful no-op or a fit of an unrelated board. Fit must not change source,
journal, selection, layout or any other pane's camera.

## Accessibility result: a concrete implementation gap

Read the menu keyboard/paint routes and inspected the Linux publication boundary:
terminal_accessibility_bridge.rs, terminal_accessibility_platform/worker.rs and
ServiceState's path dispatch/root_child_paths in terminal_accessibility_platform/atspi.rs.
The service stores a terminal snapshot and Preferences nodes; root children are
the available terminal and Preferences root. There is no menu node collection,
menu child publication, or menu enabled/focused state in that object model.
Console announcements are consequence feedback, not accessible menu controls.
This establishes a source-level gap on this bridge, not a failed native bus test
or a claim that Datum has no accessibility.

Captured as `dat-menu-accessibility-readiness-kx7`, related to the pilot after
checking for an existing menu-accessibility issue. The original action-admission
defect remains `dat-gui-local-readiness-uev`; neither issue is closed by this review.

Existing menu keys operate after a menu opens: Up/Down visit rows, Enter activates
or refuses, Right enters a submenu, Left returns, Escape dismisses. Disabled rows
are currently reachable. Keyboard focus and exposed availability must agree;
an explanation announcement alone cannot substitute for each control's state.
The activation branches clear menus without explicitly restoring editor focus;
S03/S05 must check this in the candidate build rather than infer a pass.

The pilot must publish actual menu identity, label, availability, reason and
focused state through the native accessibility service, driven from the same
production readiness state as paint and dispatch. S05 records live accessible
tree/state observations before and after native key input, plus input/dispatch
logs and visible focus evidence. No invented test-only accessibility tree.
Loss of the accessibility service is a failed/blocked S05 proof, never N/A.
Terminal and Preferences nodes, events and focus handling must remain intact.

The existing window keyboard router does not establish a keyboard-only menubar
opening route. This candidate tests actual menu-row keyboard operation after
pointer opening, plus the independent editor F shortcut. It **does not certify
keyboard-only access to the entire application**. Adding a global menu-entry
shortcut or accepting broader accessibility requires a reviewed scope addition;
it is not smuggled in as an existing input. G02 must review this explicit limit.

## Candidate contract and input closure

The JSON is an instance of the ratified closed Contract shape, not policy,
Frontier schema 6, a registry export, Proof or Review. Its Fit handler identity
names the existing production function; G04 must derive and verify that identity
from the real shared registry when implemented. Missing consumers have null
handlers. No production registry/export currently satisfies activation.

Input roots conservatively include the complete crates tree, Cargo manifests and
lock, .cargo configuration, compiled-in menu/icon data and menu authoring CSV,
guarded build runner/policy, and the existing sanitized C01 native archive.
This covers shared camera/model/numeric code, fonts, shaders, terminfo and local
crate dependencies without guessing that a linked subsystem is irrelevant.
It deliberately also catches concurrent Preferences code edits: do not call
those unrelated to this binary's proof. Unrelated documents outside these roots
and the two authority routes do not invalidate the pilot. Any later narrowing
requires independent dependency-closure review and the PM041 owner boundary.

Use the existing archive in a fresh scratch directory, not the owner's live
Project. Build with `run_cargo_guarded.py --workload proof -- cargo build --offline
--locked -p datum-gui-app --features visual --bin datum-gui`. Record command,
toolchain, environment/build flags, actual executable hash and all relevant
dirty/new/deleted input bytes. External runtime tools/backend configuration and
isolated display/accessibility-bus setup belong in the future environment record;
no downloaded dependency or unrecorded environment equivalence is approved.
The archive supplies provenance, not reusable native result proof.

Launch also uses gui-protocol's `cli_prefix` for production reads: it accepts
`EDA_CLI_BIN`, otherwise locates a sibling datum-eda binary, otherwise falls back
to `cargo run`. Therefore build the CLI separately through the same guarded
runner: `cargo build --offline --locked -p datum-eda-cli --bin datum-eda`.
Pin `EDA_CLI_BIN` to that verified executable in the isolated proof environment;
record its executable hash/build receipt alongside the GUI receipt and reject
an unverified sibling or an implicit unguarded Cargo fallback. No CLI mutation
substitutes for native user input. Launch with `--project-root` only, not `--board`:
the companion loader then seeks a sibling of native board.json, and the archived
fixture has no board.kicad_sch. Record schematic_scene=None before S03; the empty
schematic.json alone would not prove which companion path the runtime loaded.

Each scenario specifies source/journal preservation. Units/numeric entry/grid,
library changes and undo/redo of design edits are N/A only for this read-only
pilot. Focus identity, cancellation, context scope, clean reopen and accessibility
are tested rather than waived. No Save command, pending-edit recovery, automatic
camera persistence or EDA authoring success is claimed.

## Remaining G01 handoff, deliberately not closed

The implementation window must reconcile menu protocol/runtime/render ownership
and the shared native accessibility bridge with current Preferences work.
The accessibility files are a newly identified overlap risk, not a granted file
lease. Before any edits, the implementation lane must review their own controlling
routes and agree a bounded handoff; this review does not refresh their authority.
No hook/CI change is proposed in this report; the remaining trusted-runner/hook
handoff must review the visual-truth file-lane route before editing its consumer.
Implementation and independent replay session identities and owner-controlled
trusted authority/base configuration still need an explicit handoff. G01 stays
in progress; G02 execution permission and G06 activation remain pending.
