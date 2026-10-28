# gtww — ordinary-application AT-SPI observation (radial marking-menu a11y)

Bead: `dat-mm-radial-a11y-node-coverage-gtww`
Date: 2026-10-28
Commits under review: `0600d5dd` (protocol radial nodes), `d9e3cdbc` (runtime
publish-gate fix), `bd68b60e` (test-placement repair).

## What was run (transport, verbatim)

Genuine OS-input run of the **delivered** `datum-gui` binary — no Runtime/menu/
selection-state injection. Binary rebuilt through the guarded runner carrying all
three commits (`target/debug/datum-gui`). Launched on X11 `:1` under the live
KWin/Xwayland session against a **copy** of a governed fixture
(`crates/engine/testdata/golden/text/native/text-transform-repro` → scratch
`obs-project`), with `EDA_CLI_BIN=$PWD/target/debug/datum-eda`,
`WINIT_UNIX_BACKEND=x11`, `WAYLAND_DISPLAY` unset.

Input = genuine OS events via `xdotool` (XTEST) after `windowactivate`:
- `Ctrl+A` → selection became **"Sel compound 9"** (status bar, observed), 9
  board objects (`board text 8`, `board outline 1`) — confirms real OS-input
  delivery end to end.
- **Menu key** (`NamedKey::ContextMenu`) → opened the radial marking menu
  (`open_marking_menu_for_keyboard`). Status bar **focus Overlay**. Rendered wheel
  observed (screenshot `_shot_menu_live.png`): R Rotate · D Distribute › ·
  C Duplicate · A Align › · M Move Exactly · L Lock · F Flip · D Delete ·
  Select… S, labelled `pcb.multiselect`.

Note: right-mouse-button press (the pointer invocation) did not reach the client
in this KWin/Xwayland session; the keyboard invocation (an equally ordinary path)
did, so the observation used it. `busctl tree` cannot enumerate the AT-SPI tree
(the first-party atspi server implements the `org.a11y.atspi.*` interfaces, not
recursive D-Bus `Introspectable`); traversal used `org.a11y.atspi.Accessible.
GetChildren` from the application root, which is the correct AT-SPI mechanism.

## Observed on the REAL a11y bus (org.a11y.Bus, app connection :1.639 "Datum EDA")

With the menu open, `GetChildren` on the app root returned the published menu
node (object path hex-decodes to the node id):

- `menu:marking:pcb.multiselect` — Name **"Marking menu — pcb.multiselect"**,
  role Menu, 8 children. Its 8 radial leaves (role MenuItem):

  | slot | Name | state word | Description |
  |------|------|-----------|-------------|
  | N  | Rotate       | 1107298304 | This action is unavailable in this build. |
  | E  | Move Exactly | 1107298304 | move-exactly with delta |
  | S  | Delete       | 1107298304 | This action is unavailable in this build. |
  | W  | Align        | 1124075776 | Opens the Align actions |
  | NE | Duplicate    | 1107298304 | duplicate selection |
  | SE | Flip         | 1107298304 | This action is unavailable in this build. |
  | SW | Lock         | 1107298304 | This action is unavailable in this build. |
  | NW | Distribute   | 1124079872 | Opens the Distribute actions |

  The two submenu parents (Align, Distribute) carry the higher state word (the
  enabled/sensitive bits set) and the "Opens the … actions" description =
  **available/navigable**; every other leaf carries the lower state word and the
  truthful **"This action is unavailable in this build."** reason = **inert,
  available=false**. This is exactly gtww's truthful inert-pilot projection,
  reachable by a screen reader in the ordinary running app.

- Sibling `menu:native-select:93000000-…0001` — Name **"Select"** (the
  selection-list menu, present because this ordinary invocation ran with
  `selection = Some`).

- After **Escape** (dismiss): `GetChildren` on the root shows **0** menu nodes —
  the published inventory clears on close.

- **No writes:** the four project design files (`project.json`, `board/board.json`,
  `schematic/schematic.json`, `rules/rules.json`) were byte-identical to the
  fixture before and after (sha256) — the shell invoked nothing.

## Honest scope of the claim (per PM / audit bound)

Two complementary pieces, not conflated:
- **(a) ordinary AT-SPI observation** (this record): the radial marking-menu
  nodes reach the real a11y bus in normal use, with truthful labels, disabled
  states, submenu parents, the native-select sibling, and dismissal-clears — and
  no writes. This run invoked with `selection = Some` (binding basis present,
  `runtime_selection_menu.rs:59`); the internal `MarkingMenuState.selection`
  Option was **not** observed, and this run makes **no** selection=None-specific
  empirical claim (the status-bar Sel count measures selected-object count, a
  different value that cannot establish the Option).
- **(b) selection=None publication/clear**: proved separately by the committed
  focused regression `marking_a11y_tests::radial_marking_menu_without_selection_
  list_publishes_through_bridge`, which drives the real
  `menu_accessibility_projection` → `LinuxTerminalAccessibilityBridge::
  without_platform().update_menus` with `selection = None` and asserts the radial
  root + inert Rotate + navigable Align publish, then clear.

Together: the runtime gate (`d9e3cdbc`) makes gtww's radial nodes reachable at
runtime in both the ordinary (selection=Some) and the selection=None cases.
