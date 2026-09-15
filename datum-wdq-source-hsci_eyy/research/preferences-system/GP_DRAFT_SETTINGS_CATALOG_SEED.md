# Draft Global Preferences Registry Seed

> **Status:** Draft intake only — not ratified, not a descriptor catalog, and
> not implementation authority.
>
> **Source:** `docs/gui/prototypes/preferences-window.html`, prototype series
> `2e7ceb1` through `353ff27`.
>
> **Tracker:** `dat-global-preferences-engine-qcv`.

## Purpose and extraction boundary

This file absorbs the working prototype's declared settings catalog into a
searchable registry seed without converting prototype clay into specification.
The count-bearing catalog was materialized at commit `914f23c`: 60 v1
candidates and 59 planned candidates. The labels below preserve that snapshot
exactly enough for later descriptor review; spelling, grouping, defaults,
scope, class, eligibility, schema, and inclusion remain undecided.

Later prototype commits add, relocate, or decompose visible rows while retaining
the 60+59 banner. Those deltas are recorded after the seed lists and must be
reconciled only after the owner declares the prototype settled. They do not
silently change the 119-entry count.

This seed does **not**:

- ratify any candidate as a preference or as v1;
- turn actions, read-only mirrors, Project facts, or session state into stored
  preferences;
- select a default, source scope, management eligibility, or migration;
- extract GP-C03/GP-C05 clauses from the unsettled prototype;
- authorize implementation or a dependency.

## Draft v1 candidates — 60

| # | Prototype section | Draft candidate label |
|---:|---|---|
| 1 | Appearance | Console feedback duration |
| 2 | Appearance | Reduced motion |
| 3 | Appearance | High contrast and non-color cues |
| 4 | Appearance | Layer color scheme |
| 5 | Appearance | Per-object opacity |
| 6 | Appearance | Inactive-layer dim factor |
| 7 | Appearance | Pad outline mode |
| 8 | Appearance | Ghost via through pad |
| 9 | Appearance | Rounded track corners |
| 10 | Workspace | Display units |
| 11 | Workspace | Grid snap on/off |
| 12 | Workspace | Grid size presets |
| 13 | Workspace | Snap capture radius |
| 14 | Workspace | Fine-grid divisor |
| 15 | Workspace | Object snap types |
| 16 | Workspace | Grid mark style |
| 17 | Workspace | Crosshair style |
| 18 | Workspace | Keyboard bindings (keymap) |
| 19 | Workspace | Dismissed warnings restore |
| 20 | PCB/Board | Default route profile |
| 21 | PCB/Board | Per-net color application mode |
| 22 | PCB/Board | Global airwire color |
| 23 | PCB/Board | Curved airwires |
| 24 | PCB/Board | Show selected ratsnest only |
| 25 | PCB/Board | Airwires on hidden layers |
| 26 | PCB/Board | Viewport airwire culling |
| 27 | PCB/Board | Default layer visibility |
| 28 | Rules & Checks | Check profile pre-fill |
| 29 | Terminal | Terminal launch profiles |
| 30 | Terminal | Terminal theme and palette |
| 31 | Terminal | Terminal font size and ligatures |
| 32 | Terminal | Terminal cursor style |
| 33 | Terminal | Terminal bell, activity, and copy feedback |
| 34 | Terminal | Terminal scrollback limits |
| 35 | Terminal | Terminal desktop notifications |
| 36 | Terminal | Terminal key bindings |
| 37 | Terminal | Multiline paste policy |
| 38 | Terminal | OSC 52 clipboard write gating |
| 39 | Terminal | Hyperlink and file-open trust policy |
| 40 | Files & Projects | Startup and recent files |
| 41 | Files & Projects | Default file locations |
| 42 | Files & Projects | Auto-save, backup, and recovery |
| 43 | Files & Projects | New-Project seed profile selection |
| 44 | Files & Projects | New-project templates |
| 45 | Files & Projects | Title-block/sheet template seed |
| 46 | Files & Projects | Default unit system for new projects |
| 47 | Output | Default output job pre-fill |
| 48 | Output | Export destination pre-fill |
| 49 | Revision | Show/hide revision system |
| 50 | Revision | New-Project revision profile seed |
| 51 | Revision | Build hierarchy presentation seed |
| 52 | Revision | Prototype-to-production transition seed |
| 53 | Agents | Agent authority level |
| 54 | Agents | Unattended agent tool allowlist |
| 55 | Organization | Organization authority release |
| 56 | Organization | Pending organization requests |
| 57 | Organization | Managed policy package status |
| 58 | Organization | Reset, export, and import of preferences |
| 59 | Project Policy (read-only) | Active Project policy mirror |
| 60 | Project Policy (read-only) | Active Project revision policy |

## Draft planned candidates — 59

| # | Prototype section | Draft candidate label |
|---:|---|---|
| 1 | Appearance | Text render-intent backend override |
| 2 | Appearance | Editor appearance themes |
| 3 | Appearance | Anti-aliasing and rendering quality |
| 4 | Appearance | Icon theme, icon scale, and display scaling |
| 5 | Appearance | Application language |
| 6 | Appearance | Hardware graphics acceleration |
| 7 | Appearance | Large-design performance mode |
| 8 | Workspace | Mouse, wheel, and touchpad mapping |
| 9 | Workspace | Camera and pointer behavior |
| 10 | Workspace | Selection modifiers |
| 11 | Workspace | Cross-probe behavior |
| 12 | Workspace | Hover, tooltip, and HUD assistance |
| 13 | Workspace | Numeric input spin increments |
| 14 | Workspace | Double-click behavior |
| 15 | Workspace | 3D input device tuning |
| 16 | Workspace | Resource usage warnings |
| 17 | Schematic | Wire editing defaults |
| 18 | Schematic | Placement and annotation defaults |
| 19 | Schematic | Default schematic primitive properties |
| 20 | Schematic | Editing focus emphasis |
| 21 | PCB/Board | Auto-via on layer change |
| 22 | PCB/Board | Interactive routing defaults |
| 23 | PCB/Board | Routing objective profile default |
| 24 | PCB/Board | Rotation step and flip direction |
| 25 | PCB/Board | Track and arc editing mode |
| 26 | PCB/Board | Default board primitive properties |
| 27 | PCB/Board | Polygon repour automation |
| 28 | PCB/Board | Net name and clearance display |
| 29 | PCB/Board | Displayed origin and axis directions |
| 30 | Rules & Checks | Live DRC while editing |
| 31 | Rules & Checks | DRC violation overlay style |
| 32 | Terminal | Terminal graphics policy |
| 33 | Library | Library pools and search order |
| 34 | Library | IPC footprint naming/basis default |
| 35 | Library | Field name templates |
| 36 | Library | Supplier data and currency |
| 37 | Library | Library browser zoom behavior |
| 38 | Files & Projects | Default drafting standard seed |
| 39 | Files & Projects | Multi-instance file locking |
| 40 | Files & Projects | Helper applications |
| 41 | Files & Projects | Update checks |
| 42 | Files & Projects | Cache maintenance |
| 43 | Files & Projects | 3D model search paths |
| 44 | Output | Publishing destinations |
| 45 | Revision | Provisional watermark presentation |
| 46 | Revision | Git and offline exchange seed |
| 47 | Revision | Local history and session journal retention |
| 48 | Revision | Version control integration (machine-side) |
| 49 | Revision | User identity for provenance |
| 50 | Agents | Persistent agent project configuration |
| 51 | Agents | Scripting and agent API access |
| 52 | Organization | Organization-managed pins and defaults |
| 53 | Organization | Title-block scale fraction (per-firm) |
| 54 | Organization | Network access toggles |
| 55 | Organization | Telemetry and crash reporting opt-in |
| 56 | Project Policy (read-only) | ISO revision/status projection detail |
| 57 | Project Policy (read-only) | Successor-work / earlier-control adoption |
| 58 | Project Policy (read-only) | Shipped check profile set |
| 59 | Project Policy (read-only) | Library placement gate strictness |

## Post-catalog prototype delta watchlist

The current `353ff27` working surface adds or decomposes visible candidates,
including the dedicated Units page, Publish Space, Symbol Editor, and Footprint
Editor sections; quantity-specific unit overrides and precision; Project-owned
document units and drafting-standard rows; and several completeness-critic
suggestions. Some original candidates are relocated or renamed. These are
deliberately not merged into the numbered seed until prototype settlement and a
reproducible catalog reconciliation establish the resulting count and
classification.

The 2026-08-27 owner ruling ratified one later delta without settling the rest
of the catalog: Product Mechanics 036 requires a persisted machine Presentation
descriptor named **Schematic drawing theme**, selecting the governed Dark or
Light theme as a whole. It replaces the generic planned “Editor appearance
themes” candidate for the schematic scope; any Light board theme remains future
governed work. The working prototype consequently shows 61 v1 rows and 58
planned rows while retaining 119 total. No other seed classification changes.

## Later validation required by GP-C08 (historical alias GP-C05A)

For every numbered seed and post-catalog delta, the final descriptor catalog
must determine: stable key or negative classification; subsystem owner; value
schema; default/no-value; persistent and resolution scopes; Q3/Q4 management
eligibility; Project-seed eligibility and destination; apply/restart behavior;
consumers; accessibility; portability; migration/retirement; implementation
status; and exact source evidence. Actions and read-only mirrors must be
classified honestly rather than forced into preference descriptors.

<!-- EVIDENCE:GLOBAL-PREFERENCES-SPEC:DRAFT-CATALOG-SEED -->
