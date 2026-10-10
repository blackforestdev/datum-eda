# Terminal and GUI integration history

> Detailed evidence relocated from `specs/PROGRESS.md`. The tracker remains the implementation-status entry point; the generated Active Frontier alone selects work. Dated history below does not grant execution or establish a fresh PASS.

MCP schematic taxonomy note: canonical `datum.schematic.create_sheet_definition`, `create_sheet_instance`, `delete_sheet_instance`, `move_sheet_instance`, `bind_sheet_instance_port`, `unbind_sheet_instance_port`, `draw_wire`, `delete_wire`, `place_junction`, `delete_junction`, `place_noconnect`, `delete_noconnect`, `place_label`, `rename_label`, `delete_label`, `place_port`, `edit_port`, `delete_port`, `create_bus`, `edit_bus_members`, `place_bus_entry`, `delete_bus_entry`, `place_text`, `edit_text`, `delete_text`, `place_drawing_line`, `place_drawing_rect`, `place_drawing_circle`, `place_drawing_arc`, `edit_drawing_line`, `edit_drawing_rect`, `edit_drawing_circle`, `edit_drawing_arc`, `delete_drawing`, `place_symbol`, `move_symbol`, `rotate_symbol`, `mirror_symbol`, `delete_symbol`, `set_symbol_reference`, `set_symbol_value`, symbol metadata, pin override, and symbol field aliases are now native-project scoped, require explicit `path` plus sheet/object arguments, and bridge to the matching journaled `datum-eda project create-sheet-definition`, `create-sheet-instance`, `delete-sheet-instance`, `move-sheet-instance`, `bind-sheet-instance-port`, `unbind-sheet-instance-port`, `draw-wire`, `delete-wire`, `place-junction`, `delete-junction`, `place-noconnect`, `delete-noconnect`, `place-label`, `rename-label`, `delete-label`, `place-port`, `edit-port`, `delete-port`, `create-bus`, `edit-bus-members`, `place-bus-entry`, `delete-bus-entry`, `place-schematic-text`, `edit-schematic-text`, `delete-schematic-text`, `place-drawing-*`, `edit-drawing-*`, `delete-drawing`, `place-symbol`, `move-symbol`, `rotate-symbol`, `mirror-symbol`, `delete-symbol`, `set-symbol-reference`, `set-symbol-value`, `set-symbol-*`, `clear-symbol-*`, `set-pin-override`, `clear-pin-override`, `add-symbol-field`, `edit-symbol-field`, and `delete-symbol-field` commands. `create_sheet_definition` creates the definition shard and updates the schematic root definitions map in one undoable transaction; `create_sheet_instance`, `delete_sheet_instance`, and `move_sheet_instance` update the schematic root instances array through the same journaled substrate; `bind_sheet_instance_port` and `unbind_sheet_instance_port` persist parent-port bindings that make native hierarchy links real.

Terminal discovery note: production object create/update/delete templates now default to canonical proposal-first commands: `datum-eda proposal create|update|delete-output-job`, `create|update|delete-manufacturing-plan`, and `create|update|delete-panel-projection`. Agents launched from Datum discover draft-proposal authoring rather than direct production CRUD or compatibility `project ... --as-proposal` syntax.

Artifact execution note: `datum-eda artifact generate <root> --output-job <uuid>`, MCP `datum.artifact.generate` with `output_job`, and the flat MCP `run_output_job` compatibility method now execute an authored OutputJob through the canonical artifact family; production object CRUD remains on compatibility `project ...` verbs for now, but terminal discovery points those verbs at proposal creation rather than direct mutation.

Terminal discovery note: authored OutputJob execution is now advertised as `datum-eda artifact generate <root> --output-job <uuid>` and lifecycle evidence as `datum-eda artifact start-output-job-run|cancel-output-job-run` instead of the legacy `project run-output-job` / lifecycle compatibility verbs. The GUI terminal command catalog now carries structured `datum.artifact.start_output_job_run` and `datum.artifact.cancel_output_job_run` entries, and the Outputs dock exposes status-aware `START` / `CANCEL` lifecycle handoffs on OutputJob rows without adding a GUI private writer.

Terminal discovery manufacturing-set note: the GUI terminal command catalog and
`datum_terminal_context_v1.production_commands` now advertise canonical
`datum-eda artifact export-manufacturing-set "$DATUM_PROJECT_ROOT" --output-dir <dir> ...`
and
`datum-eda artifact validate-manufacturing-set "$DATUM_PROJECT_ROOT" --output-dir <dir> ...`
templates, matching the artifact CLI/MCP taxonomy instead of leaving those
production commands discoverable only through the written spec.

Terminal agent entry note: the visible GUI `AGENTS` dock entry now focuses the PTY terminal and prints shell launch guidance for installed agents (`codex`, `claude`, `aider`) using the injected `DATUM_DISCOVERY` context, rather than treating the legacy assistant lane as the canonical agent authority.

GUI agent/terminal convergence note: terminal activity summary selection now
keeps focus in the PTY terminal lane and appends a terminal-visible selection
note instead of activating or writing to the legacy assistant transcript. Closing
the active terminal tab now refreshes the surviving active session's
`DATUM_DISCOVERY` / `.datum/gui-terminal-context.json` compatibility alias so
the visible `AGENTS` launcher and terminal-launched agents do not inherit stale
context from a terminated tab. `scripts/check_gui_agent_terminal_convergence.py`
is wired into the full drift gate and locks the current convergence rules:
`AGENTS` remains a terminal prefill entry point, terminal activity selection
must not focus `DockTab::Assistant`, and terminal tab close must refresh the
surviving session context alias. Runtime context refresh now also updates the
in-memory `TerminalLaunchContext`, so newly spawned or restarted PTY sessions
inherit the latest selection/cursor/dock context rather than stale launch-time
state.

GUI terminal context visibility note: GUI-written terminal discovery files
(`.datum/terminal-contexts/<session>.json`,
`.datum/gui-terminal-context.json`, and `.datum/tool-sessions/<session>.json`)
use same-directory atomic temp-file writes plus rename, and lifecycle rewrites
use the same path. These reconstructible process-session snapshots do not force
storage-device flushes on shell launch; canonical design mutations retain their
separate journal fsync contract. Runtime context refresh failures are not
silent: they surface in the terminal lane so agents are not left with stale or
partially-written `$DATUM_DISCOVERY` state without visible diagnostics. Focused
regressions repeatedly rewrite context JSON and lifecycle state while proving
the files remain parseable and temp files do not leak.

GUI terminal session context note: `$DATUM_DISCOVERY` now includes a
`terminal_sessions` summary derived from the protocol-backed terminal lane.
The envelope exposes the active session id, active label/status/attachment,
tab count, terminal dimensions, active activity summary, and per-tab session
lineage/event-log/activity metadata. Focused regression coverage drives the
summary from a real `TerminalSessionRegistry` through `ReviewWorkspaceState`
into `.datum/gui-terminal-context.json`, so terminal-launched agents see the
same active/detached/restarted session shape as the GUI dock.

GUI/CLI active context command note: terminal discovery now includes
`active_context_commands`, a small set of catalog-rendered commands for the
currently focused production/check context. When the focused artifact or latest
check-run id exists, agents receive concrete `datum-eda artifact
show|files|preview|validate`, `datum-eda check show`, and `datum-eda check
repair-standards` commands with session-specific IDs already bound. When the GUI
selection is a CheckFinding, the same active context now also exposes
fingerprint-bound `datum-eda check waive` and `datum-eda check accept-deviation`
commands with only the rationale left as a deliberate placeholder; when focus or
selection data is absent the fields remain `null` rather than exposing
placeholder commands. GUI-written terminal discovery now resolves
`accepted_transaction_tip` from the project journal and binds `journal_show_tip`
before a CLI refresh, while CLI `context get|refresh` rebuilds the same active
command fields from the persisted discovery envelope, resolver-derived context,
and preserved selection context, so refreshed terminal-launched agents do not
lose journal or finding-level handoffs. Regression coverage verifies focused,
selected-finding, refreshed, and empty-context cases.

Embedded assistant bridge retirement note: the GUI runtime no longer imports,
spawns, polls, or writes to the former embedded assistant bridge subprocess.
`crates/gui-app/src/assistant_bridge.rs` and `scripts/datum_assistant_bridge.py`
were removed, while the visible `AGENTS` entry now preloads terminal-launched
`codex`/`claude` workflows instead of creating a parallel assistant authority.
The compatibility `DockTab::Assistant` state is terminal-only: it renders the
terminal lane, does not accept text input, has no transcript renderer, and no
longer has protocol-backed `AssistantMessage`, `AssistantLaneState`, or
`WorkspaceUiState::assistant` payloads. The GUI convergence guard now fails if
retired bridge artifacts, runtime ownership markers, assistant transcript input,
assistant-lane rendering, or protocol assistant-state structs reappear. The
bottom dock now renders `AGENTS` as an inactive launcher button rather than a
second active command surface, and the terminal lane explicitly states that
agent launches prefill `codex`/`claude` shell commands into the PTY.

PTY terminal screen note: GUI terminal parsing now handles CSI insert-character (`CSI @`), delete-character (`CSI P`), and erase-character (`CSI X`) operations without moving the cursor, with regressions proving blank-cell insertion before overwrite, shifted-row deletes, and fixed-cell blanking before subsequent printable overwrites.

PTY terminal screen note: GUI terminal parsing now captures OSC `0`/`1`/`2`
shell title updates into `TerminalLaneState.title`, renders the active
PTY-provided session label in the terminal dock header, and continues
swallowing unsupported OSC payloads without visible byte leakage. Focused
regressions cover BEL-terminated, ST-terminated, icon-title, split OSC title
updates, and unsupported OSC commands that must not overwrite the active title.

PTY terminal context note: GUI terminal parsing now captures OSC `7`
current-directory updates from `file://host/path` URIs into
`TerminalLaneState.current_working_directory`, percent-decodes the path, renders
the same value in the terminal dock header as `CWD`, and projects it into
terminal context JSON as `terminal_sessions.active_working_directory` for
terminal-launched agents. Unsupported OSC 7 URI schemes are swallowed without
changing the tracked directory.

PTY terminal screen note: bare BEL (`0x07`) now increments
`TerminalLaneState.bell_count` without visible byte leakage, and the terminal
dock surfaces the current alert count in the session header. OSC BEL
terminators remain control-string terminators and do not increment the alert
counter.

PTY terminal screen note: GUI terminal parsing now supports `ESC M` Reverse Index, with regressions proving ordinary cursor-up behavior away from the top margin and scroll-region-only downward scrolling when invoked at the active top margin.

PTY terminal screen note: GUI terminal parsing now supports CSI next-line (`CSI E`) and previous-line (`CSI F`) cursor controls, with regressions proving default and explicit row counts reset the cursor to column zero and extend terminal rows when moving downward.

PTY terminal screen note: GUI terminal parsing now supports CSI vertical-position-absolute (`CSI d`), vertical-position-relative (`CSI e`), and vertical-position-backward (`CSI k`) controls, with regressions proving these preserve the current column, create target rows when moving downward, and saturate at the top row when moving backward.

PTY terminal screen note: GUI terminal parsing now supports CSI insert-line (`CSI L`) and delete-line (`CSI M`) controls, with regressions proving row shifts stay bounded to the active full-screen or scroll-region range and preserve rows outside that region.

PTY terminal screen note: GUI terminal parsing now supports CSI scroll-up (`CSI S`) and scroll-down (`CSI T`) controls, with regressions proving full-screen and scroll-region bounded shifts clear introduced rows while preserving rows outside the active region.

PTY terminal screen note: GUI terminal parsing now uses default eight-column horizontal tab stops and supports CSI forward-tab (`CSI I`) plus backtab (`CSI Z`), with regressions proving forward-tab alignment, repeated forward-tab counts, prior-stop rewrites, and saturation at column zero.

PTY terminal screen note: GUI terminal parsing now supports CSI repeat
preceding character (`CSI b`). The parser tracks the last printable character
and routes repeats through the same cursor, autowrap, and scroll-region writer
used for normal PTY bytes, with regressions proving repeat counts, no-op
behavior before any printable character, and terminal-width wrapping.

PTY terminal screen note: GUI terminal parsing now supports `ESC D` Index and
`ESC E` Next Line. Both controls share the same scroll-region-aware linefeed
path as printable wrapping; Index preserves the active column, while Next Line
resets to column zero. Focused regressions prove normal movement and bottom
scroll-region scrolling.

PTY terminal screen note: GUI terminal parsing now treats ASCII vertical tab
(`VT`, `0x0B`) and form feed (`FF`, `0x0C`) as line-feed controls that preserve
the active column, matching terminal emulator behavior for legacy line-advance
controls without leaking visible replacement bytes.

PTY terminal screen note: GUI terminal parsing now supports `ESC c` terminal
reset. The reset clears visible rows, cursor position, saved cursor,
alternate-screen state, scroll region, pending wrap, and repeat-character state,
with regressions proving post-reset writes do not restore stale state.

PTY terminal screen note: GUI terminal parsing now swallows VT charset
designation sequences such as `ESC ( B` and `ESC ) 0`. Focused regressions prove
whole and split charset-designation sequences do not leak designation bytes into
visible terminal rows. It also supports DEC screen alignment (`ESC #8`) by
filling the visible protocol grid with unstyled `E` cells while keeping
unsupported `ESC #` intermediate controls non-leaking.

PTY terminal screen note: GUI terminal parsing now swallows non-printing
ST-terminated control strings for DCS (`ESC P`), SOS (`ESC X`), PM (`ESC ^`),
and APC (`ESC _`), plus two-byte `ESC #` intermediate sequences. Focused
regressions prove whole and split control strings, split ST terminators, and
split `ESC #` sequences do not leak payload/final bytes into visible terminal
rows.

PTY terminal screen note: GUI terminal parsing now supports xterm private
alternate/cursor modes `CSI ?47h/l`, `CSI ?1047h/l`, and `CSI ?1048h/l` in
addition to the existing `CSI ?1049h/l` alternate-screen path. The parser also
tracks bracketed paste mode via `CSI ?2004h/l`; terminal clipboard paste wraps
payload bytes in `CSI 200~` / `CSI 201~` when the active PTY requests it, and
terminal reset clears that mode. Focused regressions prove legacy `47`, modern
`1047`, and full save/restore `1049` restore the main buffer, `1048`
saves/restores cursor position without buffer switching, split alternate-screen
private-mode sequences do not leak bytes into terminal rows, and bracketed paste
state is tracked without visible escape leakage.

PTY terminal screen note: GUI terminal parsing now tracks xterm focus-event
reporting mode (`CSI ?1004h/l`) in `TerminalLaneState.focus_event_reporting`
and surfaces the active mode in the terminal dock header. While active, GUI
window focus changes now send xterm focus-in/focus-out controls (`ESC [ I` /
`ESC [ O`) to the attached PTY without marking production refreshes or warning
for detached sessions. Focused regressions prove enable, disable, split
private-mode input, terminal reset behavior, and focus-report byte selection do
not leak bytes into terminal rows.

PTY terminal screen note: GUI terminal parsing now tracks xterm mouse reporting
mode negotiation (`CSI ?1000/1002/1003h/l`) and mouse coordinate encodings
(`CSI ?1005/1006/1015h/l`) in `TerminalLaneState.mouse_reporting_mode` and
`mouse_coordinate_encoding`, then surfaces the active mode in the terminal dock
header. GUI terminal clicks and wheel input now emit classic/default X10 mouse
reports (`ESC [ M Cb Cx Cy`) when an attached PTY requests mouse reporting
without an extended coordinate encoding, UTF-8 X10-shape reports with UTF-8
encoded coordinate codepoints when UTF-8 encoding is active, URXVT reports
(`CSI Cb ; Cx ; Cy M`) when URXVT encoding is active, and xterm SGR mouse
reports (`CSI < Cb ; Cx ; Cy M/m`) when SGR encoding is active, using
protocol-visible terminal geometry in all emitted paths. `button_event` mode
reports pointer motion while a terminal mouse button is held; SGR `any_event`
mode also reports terminal pointer motion without a held button. Ordinary dock
scrolling/click actions remain in effect when the PTY has not requested mouse
reporting. Focused regressions prove mode precedence, encoding precedence,
disable/reset behavior, split private-mode input, and SGR/X10/UTF-8/URXVT mouse
byte selection do not leak bytes.

PTY terminal screen note: GUI terminal parsing now answers normal Device Status
Report queries from the PTY. `CSI 5n` replies with terminal-ready status
(`ESC [ 0 n`), and `CSI 6n` replies with the current one-based cursor position
(`ESC [ row ; col R`) while private `CSI ?6n` replies with the DEC private
cursor-position form (`ESC [ ? row ; col R`) without leaking query bytes into
visible terminal rows.
The same response path now answers primary Device Attributes (`CSI c`) and
DECID (`ESC Z`) with `ESC [ ? 1 ; 2 c`, and secondary Device Attributes
(`CSI > c`) with `ESC [ > 0 ; 0 ; 0 c`. The response path is owned by the active
`TerminalSessionRegistry` slot so each PTY tab replies from its own screen
state.

PTY terminal geometry note: GUI terminal parsing now answers xterm text-area
size reports. `CSI 18t` replies with `ESC [ 8 ; rows ; columns t` using the
active `TerminalLaneState.rows` / `columns`, so shell applications can query the
same protocol-visible geometry that Datum applies to the PTY. `CSI 19t` reports
the same active row/column grid as full-screen character size because Datum's
embedded PTY surface does not expose a larger enclosing terminal desktop.

PTY terminal geometry note: GUI terminal parsing now answers xterm text-area
pixel and cell-size reports. `CSI 14t` derives total pixel size from the active
row/column grid using stable 16x8 terminal cells, `CSI 15t` returns the same
dimensions as root/screen pixel size because Datum's embedded PTY surface has no
larger enclosing terminal desktop, and `CSI 16t` reports that same cell size
directly, avoiding platform font metric dependencies while preserving
deterministic terminal-query behavior.

PTY terminal status note: GUI terminal parsing now answers xterm window-state
and position queries. `CSI 11t` replies with `ESC [ 1 t` for the active PTY
session, `CSI 13t` returns a deterministic `0,0` top-left embedded-terminal
position, and split query bytes remain buffered without visible leakage.

PTY terminal status note: GUI terminal parsing now answers xterm icon-label and
window-title status queries. `CSI 20t` and `CSI 21t` return ST-terminated OSC
responses derived from `TerminalLaneState.title`, while stripping control bytes
from the title payload before writing the response back to the PTY. `CSI 22t`
and `CSI 23t` now save and restore the protocol title without visible terminal
output, with split-sequence regressions proving title stack controls do not leak
bytes into terminal rows.

PTY terminal screen note: GUI terminal parsing now tracks DEC application
cursor-key mode (`CSI ?1h/l`) in
`TerminalLaneState.application_cursor_keys`; terminal key routing emits SS3
arrow/Home/End sequences (`ESC O A/B/C/D/H/F`) while active and normal CSI
sequences otherwise, and the terminal dock surfaces the active mode. Focused
regressions prove enable, disable, split private-mode input, reset clearing, and
input sequence selection.

PTY terminal screen note: GUI terminal parsing now tracks DEC application
keypad mode (`ESC =` / `ESC >`) in `TerminalLaneState.application_keypad`;
terminal key routing emits SS3 keypad sequences for physical numpad keys while
active and leaves non-numpad keys on the normal text path. The terminal dock
surfaces the active mode, and focused regressions prove enable, disable, split
escape input, reset clearing, and keypad sequence selection.

PTY terminal geometry note: GUI PTY resize now has protocol-visible geometry.
`TerminalLaneState.columns` / `rows` default to `80x24`, are updated whenever
the dock/window resizes the active PTY, survive active-session restart by
reapplying the stored size to the new PTY, and are surfaced in the terminal
dock header as `SIZE CxR`. Focused registry and render regressions prove
published geometry follows resize and restart behavior.

PTY terminal screen note: GUI terminal parsing now exposes PTY screen cursor
row/column/visibility/style through `TerminalLaneState`, supports DEC private
cursor visibility mode (`CSI ?25h/l`) plus DECSCUSR cursor shape controls
(`CSI Ps SP q`), and the dock renders the protocol cursor as block, underline,
or bar only when visible. Focused regressions prove hidden/visible cursor mode
and split cursor-shape controls do not leak escape bytes into terminal rows,
terminal reset clears cursor style, and renderer contracts honor the protocol
visibility/style fields.

PTY terminal session-history note: terminal tab protocol state now carries the
durable per-session event-log path, activity event count, and compact per-tab
activity summary, allowing GUI surfaces and terminal-launched agents to
correlate each tab with its persisted JSONL history without guessing
`.datum/tool-sessions` paths. Terminal tab activation updates the visible
activity lane from that tab's persisted summary, the dock renders an activity
badge for active tabs when no restart badge is present, and focused registry
coverage proves detach/attach lifecycle events advance the protocol history
count and summary while preserving existing context/session isolation.

PTY terminal screen note: GUI terminal parsing now accepts 8-bit C1 control
bytes for CSI, OSC, DCS/SOS/PM/APC control strings, ST, Index, Next Line, and
Reverse Index. These controls dispatch through the same screen-grid paths as
their ESC-prefixed equivalents, swallow payload bytes without visible leakage,
and preserve existing split UTF-8 decoding behavior.

PTY terminal screen note: GUI terminal parsing now supports DEC autowrap mode
`CSI ?7h/l`. Autowrap defaults enabled, can be disabled for right-margin
overwrite/clamp behavior, can be re-enabled for existing wrap semantics, is
honored by repeat-character writes, and is restored by terminal reset.

PTY terminal screen note: GUI terminal parsing now supports DEC origin mode
`CSI ?6h/l`. Cursor home and `CSI H/f` plus `CSI d` vertical absolute
positioning become relative to the active scroll-region top margin while enabled
and clamp to the bottom margin; disabling origin mode restores absolute screen
addressing.

PTY terminal screen note: GUI terminal parsing now applies combined DEC/xterm
private-mode sequences in parameter order instead of stopping at the first
matched mode. Focused regressions prove `CSI ?6;7h` enables both origin-mode
scroll-region addressing and autowrap, while `CSI ?6;7l` disables both absolute
origin remapping and autowrap right-margin wrapping.

PTY terminal screen note: GUI terminal parsing now preserves cursor position
for whole-line erase `CSI 2K`, matching terminal erase semantics used by prompt
redraws and full-line status rewrites. Focused coverage proves the next
printable character lands at the pre-erase cursor column instead of incorrectly
resetting to column zero.

PTY terminal screen note: GUI terminal parsing now preserves cursor row/column
for full-display erase `CSI 2J` and scrollback/full erase `CSI 3J` instead of
treating them as terminal reset. Focused regressions prove post-erase printable
bytes land at the preserved address on the cleared screen.

PTY terminal screen note: GUI terminal parsing now supports normal insert mode
`CSI 4h` and replace mode `CSI 4l`. Printable writes and `CSI b` repeat
characters share the same insert-aware cursor path, split mode sequences do not
leak bytes, and terminal reset clears insert mode back to replacement mode.

PTY terminal screen note: GUI terminal parsing now supports mutable horizontal
tab stops. `ESC H` sets a custom stop at the current column, `CSI g` clears the
current stop including default stops, `CSI 3g` clears all stops so HT/CHT become
no-ops until a new stop is set, and terminal reset restores default eight-column
stops. HT, `CSI I`, and `CSI Z` now all consult the same tab-stop state.

PTY terminal screen note: GUI terminal parsing now retains SGR foreground color
and bold state instead of only stripping `CSI ... m` sequences. The protocol
exposes styled terminal rows through `TerminalStyledLine` spans, the dock
renders styled spans as colored monospace runs, reset prevents style leakage to
later cells, and alternate-screen save/restore preserves styled rows.

PTY terminal screen note: known-width PTY sessions now apply terminal-grid
semantics for `CSI @` insert-character and `CSI P` delete-character. Inserted
cells shift only within the visible column width and discard overflow at the
right margin; deleted cells shift left and pad the right margin with blanks.
Known-width horizontal cursor controls now also clamp to the right margin for
`CSI C`, `CSI G`, `CSI \``, `CSI a`, and `CSI H/f` column addressing instead of
allowing cursor positions beyond the visible grid. PTY resize now also passes
row geometry into the screen parser, and known-height vertical cursor controls
clamp to the bottom margin for `CSI B`, `CSI E`, `CSI d`, `CSI e`, and
`CSI H/f` row addressing. Known-height scroll-region setup (`CSI top;bottom r`)
now clamps an oversized bottom margin to the PTY bottom row instead of growing
the screen model beyond the visible grid. Known-width erase-character (`CSI X`)
now also blanks visible cells beyond the current row text up to the right margin
instead of ignoring cells that have not yet been materialized as string content.
Known-width erase-in-line (`CSI K`) now materializes visible blank cells for
modes 0/1/2 across the bounded terminal row instead of truncating or clearing
only the row string backing store. Known-geometry erase-display (`CSI J`) now
materializes visible blank cells across bounded rows and columns for modes
0/1/2/3 while preserving cursor position. Known-height line operations
(`CSI L`, `CSI M`, `CSI S`, and `CSI T`) now use the visible PTY screen region
when no scroll region is active instead of deriving the active region only from
already-materialized backing rows.
Legacy no-width/no-height parser behavior remains unchanged for compatibility
tests.

PTY terminal UX note: terminal-focused keyboard scrollback now supports
`Shift+PageUp`, `Shift+PageDown`, `Shift+Home`, and `Shift+End` without sending
those navigation keys to the PTY. The dock renders the scrollback shortcut
beside copy/paste, while plain Home/End remain PTY control sequences.

PTY terminal input note: terminal key routing now emits xterm-compatible
navigation, editing, and function-key sequences for `Insert`, `Delete`,
`PageUp`, `PageDown`, and `F1`-`F12`. Shift-modified navigation remains reserved
for GUI scrollback controls, so native shell applications receive plain terminal
keys while Datum keeps explicit scrollback shortcuts. Alt/meta character input
now emits an ESC-prefixed byte stream like native terminal emulators, while
ordinary Ctrl character chords emit ASCII control bytes for shell/readline
apps, including `Ctrl+Space` as NUL. Ctrl+Alt character chords remain reserved
instead of being downgraded into ambiguous text bytes. `Shift+Tab` now emits
the xterm reverse-tab sequence (`CSI Z`) instead of a literal tab. Modified
arrow keys now emit xterm modifier-parameter sequences such as `CSI 1;5D` for
Ctrl+Left and `CSI 1;5H` for Ctrl+Home instead of collapsing to plain keys. Modified
Insert/Delete/PageUp/PageDown now use the same xterm tilde modifier form, such
as `CSI 6;5~` for Ctrl+PageDown. Modified `F1`-`F12` now also preserve xterm
modifier parameters, using CSI forms for modified `F1`-`F4` and tilde forms for
modified `F5`-`F12`.

PTY terminal screen note: GUI terminal parsing now retains SGR background color
and inverse-video state in protocol spans alongside foreground and bold. The
dock renders inverse/background spans with a visible color fallback while
preserving the richer metadata for future cell-background rendering.

PTY terminal screen note: GUI terminal parsing now retains extended SGR color
metadata for 256-color (`38;5;n` / `48;5;n`) and truecolor (`38;2;r;g;b` /
`48;2;r;g;b`) foreground/background spans. Malformed extended-color sequences
are ignored without clearing the active style.

PTY terminal screen note: GUI terminal parsing now retains SGR italic,
underline, and strikethrough metadata (`3`/`4`/`9`) in protocol
`TerminalStyleSpan`s and honors their reset controls (`23`/`24`/`29`) without
clearing unrelated color, bold, or inverse state.

PTY terminal screen note: GUI terminal parsing now retains SGR dim and conceal
metadata (`2`/`8`) in protocol `TerminalStyleSpan`s. SGR `22` clears both bold
and dim without touching conceal or colors, while SGR `28` clears conceal
without touching other active style metadata.

PTY terminal screen note: GUI terminal parsing now retains SGR blink metadata
(`5`/`6`) in protocol `TerminalStyleSpan`s. SGR `25` clears blink without
touching unrelated style metadata.

PTY terminal screen note: GUI terminal parsing now retains SGR overline metadata
(`53`) in protocol `TerminalStyleSpan`s. SGR `55` clears overline without
touching underline, strikethrough, or unrelated style metadata.

GUI source-shard health note: the project panel now renders resolver
source-shard attention as an explicit dirty/missing/unknown breakdown instead of
a single aggregate attention count, so users and terminal-launched agents can
distinguish stale promoted files from missing journal-recovered shards. The
protocol summary now also carries an `attention` list with shard relative path,
kind, authority, taxon, and dirty state for each non-clean shard; the GUI
renders the first attention rows in the project panel and terminal context JSON
exposes the same drilldown to agents.

GUI supervision/status note: native board scene loading now resolves the project
through `ProjectResolver` and asks the `DesignModel` for the materialized board
root shard, so the viewport reflects accepted journal state rather than only
the promoted `board/board.json` file. The protocol surface now includes
`datum_gui_supervision_snapshot_v1` on `ReviewWorkspaceState`, carrying model
revision, journal cursor/tip, source-shard summary, scene object counts, check
summary, and production/proposal/artifact counts. The Outputs dock renders that
snapshot as `ENGINE SUPERVISION`. This is a read-only supervision/status
surface; GUI edit affordances remain terminal-prefilled command handoffs until
a future direct editor path constructs and commits substrate operations with
its own gates.

Generated-evidence source-shard note: public `resolve-debug` coverage now proves
journal-recovered ZoneFill, OutputJobRun, ArtifactRun, and CheckRun evidence
retain `GeneratedEvidence` authority, concrete `ZoneFill` / `OutputJobRun` /
`ArtifactRun` / `CheckRun` taxons, and `Unknown` dirty-state metadata when
promoted `.datum/zone_fills/<uuid>.json`,
`.datum/output_job_runs/<uuid>.json`, `.datum/artifact_runs/<uuid>.json`, or
`.datum/check_runs/<uuid>.json` paths are unreadable, matching the existing
ArtifactMetadata unknown-state coverage.

GUI text edit note: selected board-text Inspector entries now prefill the PTY terminal with canonical `datum-eda project edit-board-text "$DATUM_PROJECT_ROOT" --text <uuid> ...` commands instead of opening assistant-only `/text` commands or mutating board JSON directly. Explicit center edits prefill the current value; edge/cycle controls prefill the next boolean/cycle/step value, with height steppers carrying paired `--height-nm` and `--stroke-width-nm` arguments so proportional scaling still routes through the journaled `SetBoardText` CLI path. The legacy assistant `/text` command parser and completions have been removed, and the old `gui-protocol` board-text private writer modules were deleted rather than left as dormant public helpers.

Open tracking rule:
- Add implementation evidence here before marking a substrate row complete.
- If a row is intentionally deferred, record the governance reason and the
  downstream fidelity work allowed despite the deferral.
- Do not promote legacy milestone completion to substrate readiness without a
  direct product-mechanics contract.
