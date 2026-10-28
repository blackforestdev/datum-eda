# MM-C06 — ordinary-input run of the delivered marking-menu shell

**Verdict: PASS** (after two bounded fixes this run surfaced). The delivered inert
marking-menu shell is reachable and renders legibly above the PCB through genuine
ordinary-designer OS input: `aff7059b` made the `pcb.multiselect` wheel reachable,
and `be9f709e` routed the menu to the foreground overlay stream so its opaque wedges
paint above board content. The shell fires no verb and mutates no design.

<!-- EVIDENCE:MM-C04-ORDINARY-INPUT-RUN:2026-10-27 -->

## Two defects this run surfaced (the point of an ordinary-input run) — both fixed
1. **Reachability (`aff7059b`):** `pcb.multiselect` was defined but unreachable from
   any production invocation (only tests set it); a real multi-selection never got
   the multiselect wheel. Fixed: the invocation resolves the key from the live
   selection count.
2. **Paint stream (`be9f709e`):** the marking menu emitted into `Stream::Panel`,
   which composites below/interleaved with the PCB, so over a real board the wheel
   was occluded — the empty-pane fixtures could not expose this. Fixed: the wheel now
   routes to the menu-overlay streams (`Stream::Overlay`/`OverlayText`) like the menu
   bar, painting opaque above the board. Neither the empty-pane fixture nor the
   programmatic MM-C04 proof could have caught this; the ordinary-input run did.

**Subject:** the delivered inert shell at HEAD — MM-C03 (`3541d0ea`), wedge form
(`42b3b525`), plus the multiselect-reachability fix (`aff7059b`). Binary built fresh
at HEAD with the fix (`target/debug/datum-gui`, mtime 17:46) via the guarded runner.

## Transport (stated explicitly, per the integrity bar)
Genuine OS-level input only — **no Runtime / menu / selection state was injected**
(that is what made the MM-C04 programmatic proof insufficient). The ordinary
`datum-gui` binary was launched on the real X display `:1` (winit X11 backend forced
with `WINIT_UNIX_BACKEND=x11`, `WAYLAND_DISPLAY` unset) with `EDA_CLI_BIN` pinned to
the HEAD CLI, on a **copy** of the governed canonical DOA2526 project
(test-harness/testdata/selection/s5a_v1 → scratch). Input was produced with
`xdotool` against the X server — pointer motion (XWarpPointer), mouse buttons and
keys (XTEST) — i.e. events the window system routes to the focused window exactly as
a human's would be. Screenshots are the live window pixels (`import -window <id>`),
compositor/app-presented. Observed render latency on synthetic events was handled by
a post-input pointer nudge that forces a redraw before capture; it changes nothing
about how the input itself was delivered. All screenshots referenced below are in
`scratchpad/mm-c06-run/`.

Launch command (verbatim, as executed from the repo root; the binary is the
`be9f709e` build):

```
WAYLAND_DISPLAY= WINIT_UNIX_BACKEND=x11 DISPLAY=:1 EDA_CLI_BIN=/home/bfadmin/Documents/datum-eda/target/debug/datum-eda /home/bfadmin/Documents/datum-eda/target/debug/datum-gui --project-root /tmp/claude-1000/-home-bfadmin-Documents-datum-eda/8e619a1b-92df-496e-a133-9ed9b3dd45b0/scratchpad/mm-c06-run/project
```

(The window lands wherever the bare `:1` server places it, e.g. root origin
(293,154); input coordinates are computed as `root = content + origin`, re-read per
launch with `xdotool getwindowgeometry`.)

## Scenarios

### (i) Single-object invocation → correct contextual (per-type) wheel — PASS
With nothing selected, a right-button press on a pad opened the **`pcb.pad`** wheel
(`G2-pertype-held.png` / `G2-crop.png`): the menu key label reads `pcb.pad` and the
radial wheel shows the pad verbs — Assign Net, Properties (disabled), Highlight Net,
Change Layer, Delete, Lock, Move — around the `MM` hub. The context key is resolved
from the object under the cursor, so different object types yield different wheels.

### (ii) Multi-object invocation → pcb.multiselect → Align sub-wheel → back — PASS (fix aff7059b)
A multi-selection was made with Select-All (Ctrl+A → "Compound Selection — 260 obj",
status "Sel compound 260"). A right-button click then opened the **`pcb.multiselect`**
wheel (`C2-rightclick.png`; menu label `pcb.multiselect`, items Rotate / Move Exactly
/ Delete / Align / Duplicate / Flip / Lock / Distribute). **This is the repair:**
before `aff7059b` the production resolver only ever produced per-type keys, so this
wheel was unreachable by any real multi-selection; it now resolves because the
invocation reads the live selection count (members > 1). A left-click on the W/Align
wedge opened the content-sized **Align sub-wheel** (`D1-align-subwheel.png`:
Horizontal >, Vertical >, Distribute >). A left-click on the centre back affordance
returned to the base multiselect wheel (`D2-back-to-base.png`). Full navigate in →
out confirmed.

### (iii) Disabled leaf (no verb fires) + dismiss — PASS
Every leaf is inert: focusing a disabled leaf renders the truthful reason caption
**"This action is unavailable in this build."** under the wheel
(`diag-at-400-400.png`, captured under a genuine right-press). The menu dismisses on
a click outside it (`F0-dismiss-deselect.png` → menu gone), and Escape clears the
selection (`F0c-status.png` → "Sel none"). No leaf activation produced any effect.

## Non-mutation (the core inert guarantee) — PASS
Before/after the entire real-input traversal, the **design source, journal and
revision blobs are byte-identical** (sha256 of the design+journal file set:
`9fae61f4d5d651f684cbc1a2d475ce8055a3a05799db13b0e6b4f4d935d4c239`, unchanged). The
only filesystem delta is the app's own GUI session scaffolding, which appears on any
launch and is not a design mutation:
`.datum/revision/writer.lock`, `.datum/terminal-contexts/`, `.datum/tool-sessions/`,
`.datum/gui-terminal-context.json`. So the inert shell fired no verb, authored no
operation, and wrote no journal entry across open → navigate → sub-wheel → back →
per-type → disabled-leaf → dismiss.

## Honest observations (not blocking)
- **Synthetic-input delivery was intermittent.** Fast synthetic rubber-band drags
  did not register as box-selection, and a right-click on empty space with nothing
  selected opens then immediately dismisses (correct per the shell's release-branch:
  a release outside any slot with no selection dismisses). Multi-selection was
  therefore produced with Ctrl+A and per-type with a press-hold. Whether the
  rubber-band non-registration is an OS-delivery artifact of synthetic drags or a
  product gap was not isolated here (DATUM_TRACE_CLICKS would separate them); it did
  not block any scenario and is flagged, not asserted as a defect.
- **Legibility — RESOLVED by `be9f709e`:** the initial captures showed the wheel
  washed out over board content; root-caused to the Panel-stream paint defect above.
  The recapture on the fixed build shows opaque wedges painting above the PCB:
  `R4-multiselect`/`R4-crop` — all eight multiselect labels (Rotate, Duplicate, Move
  Exactly, Flip, Delete, Lock, Align >, Distribute >) legible over the 260-object
  board; `R3-pertype`/`R3-crop` — the `pcb.pad` wheel legible over the board;
  `R6-crop` — multiselect with the Align wedge highlighted; and `S2-align-subwheel`/
  `S2-crop` — the OPEN nested Align sub-wheel (Horizontal >, Vertical >, Distribute >,
  centre `<` back) as opaque content-sized wedges over the board, reached by a real
  left-click on the Align wedge (scenario (ii)'s nested state, observed not inferred).
  Because the wedges appear
  via the same `import -window` method after the fix, the X11 backing-pixmap question
  is moot: it was a real paint-order defect, not a capture artifact.
- **Build scoping / pinned companion:** the fixed-build recapture rebuilt only
  `-p datum-gui-app` (the pinned `EDA_CLI_BIN` companion `target/debug/datum-eda`
  stayed byte-identical, sha256 `c6ab65cb…`), and the design+journal non-mutation
  still held on the fixed build (`9fae61f4…`, unchanged).
