# Datum Global Preferences V1 Descriptor Catalog

> **Status:** owner-ratified reserved V1 inventory through GP-C06, Product
> Mechanics 037, and Product Mechanics 040; production activation amended by
> the owner GP-CM01 correction of 2026-09-05. Eleven descriptors are
> production-active and 45 remain reserved candidates. Reservation is not
> implementation or product-surface authority.
>
> **Historical step alias:** GP-C05A.
>
> **Tracker:** `dat-global-preferences-engine-qcv`.
>
> **Boundary:** specification evidence only. This catalog does not authorize
> implementation, a dependency, a policy provider, or a prototype edit.

## 1. Catalog law

This catalog is the complete initial disposition of the 119-item prototype seed,
the later Units/Publish/editor deltas, and every setting-like row now drawn in the
Preferences window. An entry is production-active, a reserved V1 candidate,
explicitly deferred with no key allocated, or negatively classified. A label,
control, registration, or stored byte sequence does not become an active
preference merely because it appears in Preferences or this inventory.

Every reserved row below is descriptor schema version `1`. Its stable key has one
subsystem owner and one meaning. `DescriptorDefault` is compiled truth and is
not serialized as a user choice. Unless a row says otherwise, its persistent
scope is the machine-local user profile; its eligible resolution sources are
`DescriptorDefault`, `Installation`, `Organization`, `User`, `Session`, and
descriptor-approved `Context`; ordinary values resolve under Q4; changes apply
live; import/export is portable; and reset removes the user contribution so the
resolver exposes the next eligible contribution. All values are validated and
canonicalized before storage or use.

The visual label and one-sentence consequence in the Claude-owned window are the
accessible name and description. The row must also expose its real control,
provenance, effective state, reset availability, and explanation under GP-C05.
Boolean/enumerated state is never conveyed by color alone. Capability controls
add a protected-state cue and refusal reason. Seed controls name the destination
Project authority and disclose copy-once receipt semantics.

### 1.1 Table codes

| Code | Meaning |
|---|---|
| `P` / `W` / `C` / `PS` | `Presentation`, `WorkflowDefault`, `Capability`, or registered `ProjectPolicySeed` class |
| `R,C,Pn,L` | organization Recommend, Constrain, Pin, and Lock are descriptor-eligible, subject to the user-held Q3 authority release |
| `R,C` | Recommend and Constrain only; organization cannot govern the personal presentation choice |
| `none` | personal accessibility carve-out; no organization control |
| `U+S` | persistent User value with a nonpersistent Session override |
| `U+S+C` | User and Session plus descriptor-approved focused-Project Context |
| `seed:<authority>` | eligible for Q5 copy-once seeding into the named Project authority, with durable itemized receipt |
| `P0` / `PX` | portable, or protected/excluded from portable exchange |
| `I` / `M` / `N` | implement new, migrate a legacy input, or negative/non-descriptor |

`Last used` is an explicit enum value, not an implicit source-precedence rule.
Paths are canonical platform paths but exports use a user-approved portable path
token or omit the value; Datum never exports home-directory expansion or secrets.
Lists/maps use deterministic ordering and reject duplicate identities.
Every V1 descriptor's merge category is `Replace`: one validated contribution
is an atomic value, including structs, sets, lists, and maps. No descriptor in
this catalog declares a deterministic equal-authority join; incompatible
equal-authority controls therefore remain Q4 unresolved conflicts rather than
being combined field-by-field or by arrival order.

`ProjectPolicySeed` is a registered descriptor class, not an eligibility flag
on another class. Exactly fourteen reserved rows whose Apply column says
`seed:<authority>` below are `PS`; the eight `datum.units.*` rows are initially
production-active. Three formerly active Revision seed rows were
withdrawn by Product Mechanics 038, and the deferred AdoptedDraftingStandard
seed remains classified `PS` while its schema is unavailable. Presentation,
Capability, WorkflowDefault, and other non-`PS` keys cannot cross Q5 into
Project authority. A `PS` descriptor
accepts only DescriptorDefault, Installation, Organization, and User seed-profile
contributions; Session and Context never enter a Project seed snapshot.

## 2. Reserved V1 inventory and production-active subset

Production activation is consumer-ready, not registration-ready. The exact
initial active inventory is `datum.console.feedback_duration`,
`datum.accessibility.reduced_motion`,
`datum.accessibility.high_contrast_noncolor`, and the eight `datum.units.*`
rows in section 2.2. The other 45 rows below retain their stable identities and
schemas as reserved candidates but do not enter the production resolver,
repository, search index, or GUI until their control, consumer, effect timing,
accessibility, and proof are complete. `datum.projects.unit_policy_seed` is a
reserved internal aggregate and is excluded from the ordinary GUI pending
separate justification.

### 2.1 Appearance and shared viewport

| Stable key | Owner | Value schema; factory/no-value | Class; scopes; management | Seed/apply; consumers | Portability; migration; disposition | Evidence |
|---|---|---|---|---|---|---|
| `datum.console.feedback_duration` | GUI shell | enum `{4s,6s,10s,never}`; `6s` | P; U+S; R,C | live; Console feedback timer | P0; alias `console_duration`; M from current field | `preferences-window.html:90`; `GP_C01_INTERNAL_AUTHORITY_AUDIT.md:84-112` |
| `datum.accessibility.reduced_motion` | GUI accessibility | bool; `false` | P; U+S; none | live; all GUI/terminal animation | P0; no legacy; I | `preferences-window.html:92`; `preferences-accessibility-study.html:113-121` |
| `datum.accessibility.high_contrast_noncolor` | GUI accessibility | bool; `false` | P; U+S; none | live; GUI/terminal renderers | P0; no legacy; I | `preferences-window.html:94`; `preferences-accessibility-study.html:123-130` |
| `datum.pcb.layer_color_scheme` | PCB presentation | enum `{datum,high_contrast_mono,photonics}`; `datum` | P; U+S+C; R,C | live; board renderer | P0; no legacy; I | `preferences-window.html:96` |
| `datum.schematic.drawing_theme` | schematic presentation | enum `{dark,light}`; `dark` | P; U+S; R,C | live; schematic canvas only | P0; pre-ratification draft name `datum.schematic.theme` remains searchable only; I; PM-036 | `preferences-window.html:98`; `PRODUCT_MECHANICS_036_SCHEMATIC_DRAWING_THEMES.md:18-57` |
| `datum.pcb.object_opacity` | PCB presentation | struct `{track,via,pad,zone:u8 0..100}`; `{100,100,100,70}` | P; U+S+C; R,C | live; board renderer | P0; no legacy; I | `preferences-window.html:100` |
| `datum.pcb.inactive_layer_dim_percent` | PCB presentation | integer `0..100`; `50` | P; U+S+C; R,C | live; board renderer | P0; no legacy; I | `preferences-window.html:102` |
| `datum.pcb.pad_outline_mode` | PCB presentation | bool; `false` | P; U+S+C; R,C | live; board renderer | P0; no legacy; I | `preferences-window.html:104` |
| `datum.pcb.ghost_via_through_pad` | PCB presentation | bool; `false` | P; U+S+C; R,C | live; board renderer | P0; no legacy; I | `preferences-window.html:106` |
| `datum.pcb.rounded_track_corners` | PCB presentation | bool; `false` | P; U+S+C; R,C | live; board renderer | P0; no legacy; I | `preferences-window.html:108` |
| `datum.viewport.snap_enabled` | shared viewport | bool; `true` | W; U+S+C; R,C,Pn,L | live; schematic/PCB editors | P0; legacy hard-coded default; I | `preferences-window.html:114`; `crates/gui-protocol/src/lib.rs:528-540,1802-1810` |
| `datum.viewport.snap_capture_px` | shared viewport | integer `1..64`; `10` | W; U+S+C; R,C,Pn,L | live; snap resolver | P0; legacy code default; I | `preferences-window.html:118`; `docs/gui/DATUM_UNIVERSAL_VIEWPORT_TOOLING_SPEC.md:2285-2309` |
| `datum.viewport.fine_grid_divisor` | shared viewport | enum `{2,5,10}`; `10` | W; U+S+C; R,C,Pn,L | live; grid resolver | P0; no legacy; I | `preferences-window.html:120` |
| `datum.viewport.object_snap_types` | shared viewport | set of typed snap kinds plus `current_layer_only`; all kinds/false | W; U+S+C; R,C,Pn,L | live; snap resolver | P0; no legacy; I | `preferences-window.html:122` |
| `datum.viewport.grid_mark_style` | shared viewport | struct `{shape:cross-or-dot-or-line,size:u8,min_spacing_px:u8}`; `dot` with engine profile defaults | P; U+S+C; R,C | live; grid renderer | P0; no legacy; I | `preferences-window.html:124` |
| `datum.input.editor_keymap` | input authority | versioned map from action identity to non-exclusive gesture; Datum default | W; U+S+C; R,C,Pn,L | live after whole-map validation; editor dispatch | P0; no legacy; I | `preferences-window.html:128` |

Grid pitch/profile tables remain engine `ViewportProfile` facts. Exclusive
ratified gestures cannot be rebound by the keymap descriptor. The clay grid-set
and crosshair-persistence proposals remain deferred in §4 rather than being
silently promoted into V1.

### 2.2 Units

These eight descriptors are **Global defaults for new Projects**, not live unit
controls for an open Project. Canonical design truth remains exact signed
integer nanometers. One shared service accepts explicit unit suffixes for GUI,
CLI, and MCP; a bare number uses explicit request context or the owning
Project's effective Working Units. These descriptors never rescale stored
design data and never alter an existing Project.

| Stable key | Owner | Value schema; factory/no-value | Class; scopes; management | Seed/apply; consumers | Portability; migration; disposition | Evidence |
|---|---|---|---|---|---|---|
| `datum.units.system` | units engine | enum `{metric,imperial}`; `metric` | PS; U; R,C,Pn,L | seed:`ProjectDisplayUnits`; future-Project default only | P0; splits legacy aggregate Display units; I | `preferences-window.html:137`; `GP_SHARED_UNITS_ENGINE_REQUIREMENT.md:34-82`; PM-040 |
| `datum.units.board_length` | units engine | enum `{follow_system,mm,um,mil,inch}` with quantity validation; `follow_system` | PS; U; R,C,Pn,L | seed:`ProjectDisplayUnits`; future-Project default only | P0; split aggregate; I | `preferences-window.html` Units target; `GP_SHARED_UNITS_ENGINE_REQUIREMENT.md#typed-profile-and-resolution`; PM-040 |
| `datum.units.board_length_precision` | units engine | enum `{automatic,decimal_0,decimal_1,decimal_2,decimal_3,decimal_4,decimal_5,decimal_6,exact_nm}`; `automatic` | PS; U; R,C,Pn,L | seed:`ProjectDisplayUnits`; Board display only | P0; migrate/fan out retired `datum.units.length_precision`; I | `GP_SHARED_UNITS_ENGINE_REQUIREMENT.md#typed-profile-and-resolution`; PM-040 |
| `datum.units.drill_hole` | units engine | enum `{follow_system,mm,mil,inch}`; `follow_system` | PS; U; R,C,Pn,L | seed:`ProjectDisplayUnits`; future-Project default only | P0; split aggregate; I | `preferences-window.html` Units target; `GP_SHARED_UNITS_ENGINE_REQUIREMENT.md#typed-profile-and-resolution`; PM-040 |
| `datum.units.drill_hole_precision` | units engine | enum `{automatic,decimal_0,decimal_1,decimal_2,decimal_3,decimal_4,decimal_5,decimal_6,exact_nm}`; `automatic` | PS; U; R,C,Pn,L | seed:`ProjectDisplayUnits`; Drill display only | P0; migrate/fan out retired `datum.units.length_precision`; I | `GP_SHARED_UNITS_ENGINE_REQUIREMENT.md#typed-profile-and-resolution`; PM-040 |
| `datum.units.schematic_geometry` | units engine | enum `{follow_system,mm,mil}`; `follow_system` | PS; U; R,C,Pn,L | seed:`ProjectDisplayUnits`; future-Project default only | P0; split aggregate; I | `preferences-window.html` Units target; `GP_SHARED_UNITS_ENGINE_REQUIREMENT.md#typed-profile-and-resolution`; PM-040 |
| `datum.units.schematic_geometry_precision` | units engine | enum `{automatic,decimal_0,decimal_1,decimal_2,decimal_3,decimal_4,decimal_5,decimal_6,exact_nm}`; `automatic` | PS; U; R,C,Pn,L | seed:`ProjectDisplayUnits`; Schematic display only | P0; migrate/fan out retired `datum.units.length_precision`; I | `GP_SHARED_UNITS_ENGINE_REQUIREMENT.md#typed-profile-and-resolution`; PM-040 |
| `datum.units.angle_precision` | units engine | enum `{decimal_0,decimal_1,decimal_2,decimal_3}`; `decimal_1` | PS; U; R,C,Pn,L | seed:`ProjectDisplayUnits`; decimal-degree display/input only | P0; migrate supported decimal-degree draft values; preserve/refuse DMS or radians; I | `GP_SHARED_UNITS_ENGINE_REQUIREMENT.md#v1-angle-service`; PM-040 |

The retired `datum.units.length_precision` and `datum.units.angle_format` names
are migration inputs, not active descriptors. The former fans a valid value out
atomically to the three precision descriptors; the latter migrates only supported
decimal-degree values. Unsupported draft angle notations remain preserved
evidence and cannot silently become a different V1 preference.

The eight typed `datum.units.*` seed descriptors compose the default
`ProjectDisplayUnits` snapshot. `datum.projects.unit_policy_seed` is the
explicit aggregate override: an eligible non-absent contribution to it wins for
the seed transaction; when it has no contribution, the snapshot is composed
from the eight typed unit-seed descriptors. Absence never masquerades as a
contribution, so the aggregate factory/no-value does not shadow more specific
chosen unit seeds. The receipt records either the aggregate source or every
composed source and effective value. The aggregate is not an ordinary GUI row;
the eight typed controls are the complete user-facing Global Units surface.

The eight rows have one Global role: define defaults for future Projects. At New
Project they are eligible inputs to one immutable `ProjectDisplayUnits`
snapshot copied by Project mutation authority with a receipt. Thereafter that
Project-owned schema is exposed as **Project Working Units** and governs editor
display and contextual bare input. Existing Projects never follow a later
Global change. Publish/document units remain separate Project/document
authority. CLI/MCP bare expressions require explicit context or a Project and
field; interchange never reads these preferences. Exact resolution, token
mapping, decimal-degree angle limits, lossless editing, scalar-expression
grammar, aggregate composition, and cross-surface proof are
governed by Product Mechanics 040 and the industrial execution contract in
`GP_SHARED_UNITS_ENGINE_REQUIREMENT.md`.

### 2.3 PCB, checks, and publish-space defaults

| Stable key | Owner | Value schema; factory/no-value | Class; scopes; management | Seed/apply; consumers | Portability; migration; disposition | Evidence |
|---|---|---|---|---|---|---|
| `datum.pcb.route_profile` | PCB routing | enum `{conservative,balanced,high_density}`; `conservative` | W; U+S+C; R,C,Pn,L | live pre-fill; route proposal; operation still asks | P0; no legacy; I | `preferences-window.html:154` |
| `datum.pcb.net_color_application` | PCB presentation | enum `{none,ratsnest,copper_ratsnest,all}`; `ratsnest` | P; U+S+C; R,C | live; board renderer | P0; no legacy; I | `preferences-window.html:156` |
| `datum.pcb.global_airwire_color` | PCB presentation | canonical RGBA color; Datum palette token | P; U+S+C; R,C | live; airwire renderer | P0; no legacy; I | `preferences-window.html:158` |
| `datum.pcb.curved_airwires` | PCB presentation | bool; `false` | P; U+S+C; R,C | live; airwire renderer | P0; no legacy; I | `preferences-window.html:160` |
| `datum.pcb.selected_ratsnest_only` | PCB presentation | bool; `true` | P; U+S+C; R,C | live; selection/airwire renderer | P0; no legacy; I | `preferences-window.html:162` |
| `datum.pcb.airwires_hidden_layers` | PCB presentation | enum `{visible_layers,all_layers}`; `visible_layers` | P; U+S+C; R,C | live; airwire renderer | P0; no legacy; I | `preferences-window.html:164` |
| `datum.pcb.viewport_airwire_culling` | PCB presentation | bool; `false` | P; U+S+C; R,C | live; airwire renderer/performance | P0; no legacy; I | `preferences-window.html:166` |
| `datum.checks.profile_prefill` | checks engine | enum `{full,fast,last_used}`; `last_used` | W; U+S+C; R,C,Pn,L | live pre-fill; `run_check`; operation still names profile | P0; no legacy; I | `preferences-window.html:189` |
| `datum.publish.title_block_template_seed` | documentation | resolvable template-set identity; Datum factory set | PS; U; R,C,Pn,L | seed:`ProjectDocumentTemplates`; New Project only | P0, missing identity refuses seed; no legacy; I | `preferences-window.html:177-178`; `PRODUCT_MECHANICS_020_PAPER_SPACE_AND_VIEWPORTS.md:18-55` |
| `datum.publish.sheet_format_seed` | documentation | struct `{size,orientation}`; A3/landscape | PS; U; R,C,Pn,L | seed:`ProjectDocumentTemplates`; New Project only | P0; no legacy; I | `preferences-window.html:178-179` |
| `datum.publish.scale_fraction_style_seed` | documentation | enum `{one_to_n,n_over_one,custom_pattern}`; `one_to_n` | PS; U; R,C,Pn,L | seed:`AdoptedDraftingStandard`; New Project only | P0; moved from planned organization row; I | `preferences-window.html:179-180`; `PRODUCT_MECHANICS_035_ADOPTED_DRAFTING_STANDARD_AUTHORITY.md:18-51` |
| `datum.publish.viewport_creation_prefill` | documentation | enum `{on_demand,on_demand_remember_style}`; `on_demand` | W; U+S+C; R,C,Pn,L | live pre-fill; publish viewport creation | P0; operation remains explicit; I | `preferences-window.html:181-182` |
| `datum.publish.publish_set_naming` | documentation | struct `{mode:ask-or-pattern,pattern}`; pattern `{project}-{set}` | W; U+S+C; R,C,Pn,L | live pre-fill; publish-set creation | P0; draft name `datum.publish.set_name_prefill` remains searchable only; operation still asks; I | `preferences-window.html:182-183` |

The airwire-culling control and its description both state factory `Off` after
Claude commit `d414462`; the descriptor default is therefore `false` without a
remaining mismatch.

### 2.4 Terminal

| Stable key | Owner | Value schema; factory/no-value | Class; scopes; management | Seed/apply; consumers | Portability; migration; disposition | Evidence |
|---|---|---|---|---|---|---|
| `datum.terminal.launch_profiles` | terminal session | versioned named profiles `{executable,argv,cwd,environment}`; platform default shell | C; U+S; R,C,Pn,L | next terminal launch; PTY/session launcher | PX; replace process-launch inputs only when explicitly selected; I | `preferences-window.html:193`; `GP_C01_INTERNAL_AUTHORITY_AUDIT.md:180-200` |
| `datum.terminal.theme` | terminal presentation | enum/palette `{datum_dark,high_contrast}`; `datum_dark` | P; U+S; R,C | live; terminal renderer | P0; current session cycle remains Session; I | `preferences-window.html:195` |
| `datum.terminal.text_rendering` | terminal presentation | struct `{zoom:60..200 step10,ligatures:bool}`; `100,false` | P; U+S; R,C | live; terminal renderer; typeface not configurable | P0; no legacy; I | `preferences-window.html:197` |
| `datum.terminal.cursor` | terminal presentation | struct `{shape:block-or-bar-or-underline,blink:bool}`; block/true | P; U+S; R,C | live; terminal renderer | P0; no legacy; I | `preferences-window.html:199` |
| `datum.terminal.feedback` | terminal presentation | struct `{bell:visual-or-audible-or-both-or-none,activity:bool,copy_confirmation:bool}`; visual/true/false | P; U+S; R,C | live; terminal/desktop shell | P0; retired `terminal.legacy_bell_mode` only after a typed migration is declared; I | `preferences-window.html:201`; `preferences-window.html:291-292` |
| `datum.terminal.scrollback` | terminal resource policy | struct `{max_lines:100..100000,max_mib:1..64}`; 50000/64 | C; U+S; R,C,Pn,L | live with bounded trim; terminal VT/state | PX; no legacy; I | `preferences-window.html:203` |
| `datum.terminal.notifications` | terminal capability | enum `{off,unfocused,always}`; `unfocused` | C; U+S; R,C,Pn,L | live; desktop notification adapter | PX; migrate `DATUM_TERMINAL_NOTIFICATIONS` only with explicit provenance; M | `preferences-window.html:205`; `GP_C01_INTERNAL_AUTHORITY_AUDIT.md:206-226` |
| `datum.terminal.keymap` | terminal input | versioned action-to-gesture map; Datum default | W; U+S; R,C,Pn,L | live after whole-map validation; terminal dispatcher | P0; no legacy; I | `preferences-window.html:207` |
| `datum.terminal.multiline_paste` | terminal security | enum `{ask,allow,block}`; `ask` | C; U+S; R,C,Pn,L | live; paste gate | PX; no legacy; I | `preferences-window.html:209` |
| `datum.terminal.osc52_write` | terminal security | enum `{ask,allow,block}`; `ask` | C; U+S; R,C,Pn,L | live; OSC 52 gate; reads remain denied | PX; no legacy; I | `preferences-window.html:211` |
| `datum.terminal.open_target_policy` | terminal security | enum `{ask,trusted_only,never}`; `ask` | C; U+S; R,C,Pn,L | live; hyperlink/file opener | PX; no legacy; I | `preferences-window.html:213` |

Portable sources cannot supply Capability-class values. Terminal launch exports
may carry a profile shell only in an explicitly protected machine transfer;
portable exchange omits executable, environment, credentials, and expanded cwd.

### 2.5 Files, output, revision, and agents

| Stable key | Owner | Value schema; factory/no-value | Class; scopes; management | Seed/apply; consumers | Portability; migration; disposition | Evidence |
|---|---|---|---|---|---|---|
| `datum.projects.startup_view` | application shell | enum `{start_page,last_session,empty}`; `start_page` | W; U+S; R,C,Pn,L | next launch; shell/start page | P0; draft name `datum.files.startup_mode` remains searchable only; recent list remains restartable state; I | `preferences-window.html:239`; `start-page-study.html:184-249` |
| `datum.files.default_locations` | file services | typed map `{projects,libraries,output_jobs,new_project}` to paths; platform Documents/Datum | W; U+S; R,C,Pn,L | live pre-fill; file pickers | portable path tokens only; no legacy; I | `preferences-window.html:241` |
| `datum.files.autosave` | document persistence | struct `{interval:off-or-5m-or-10m-or-30m,retained_versions,backup_age,recovery,reminder}`; `10m` plus bounded engine defaults | W; U+S+C; R,C,Pn,L | live schedule; document persistence | P0 excluding recovery payloads; no legacy; I | `preferences-window.html:243`; `GP_C01_INTERNAL_AUTHORITY_AUDIT.md:282-296` |
| `datum.projects.seed_profile` | Project genesis | resolvable profile identity; Datum factory | PS; U; R,C,Pn,L | seed:`ProjectSeedSnapshot`; New Project only | P0, missing identity refuses selection; no legacy; I | `preferences-window.html:245`; `GP_C03_PROJECT_SEED_AND_CONTEXT_DECISION_PACKET.md:132-169` |
| `datum.projects.template_set` | Project genesis | resolvable template identity or none; Datum starter | PS; U; R,C,Pn,L | seed:`ProjectTemplateCopy`; New Project only | P0; draft name `datum.projects.template_seed` remains searchable only; I | `preferences-window.html:247` |
| `datum.projects.unit_policy_seed` | Project genesis/units | optional aggregate `UnitsProfile` with system, three quantity-specific unit/precision pairs, and decimal-degree precision; absent | PS; U; R,C,Pn,L | seed:`ProjectDisplayUnits`; explicit aggregate wins, otherwise compose typed unit seeds | P0; migrate only through the controlling Units contract; I | `preferences-window.html:249`; `GP_C03_PROJECT_SEED_AND_CONTEXT_DECISION_PACKET.md:132-169`; `GP_SHARED_UNITS_ENGINE_REQUIREMENT.md#typed-profile-and-resolution` |
| `datum.output.job_prefill` | output orchestration | enum `{ask,last_used}`; `ask` | W; U+S+C; R,C,Pn,L | live pre-fill; Generate; operation still asks | P0; output job stays Project fact; I | `preferences-window.html:258` |
| `datum.output.destination_prefill` | output orchestration | enum `{ask,project_outputs}`; `ask` | W; U+S+C; R,C,Pn,L | live pre-fill; export; operation still asks | P0; no legacy; I | `preferences-window.html:260` |
## 3. Negative classifications

These setting-like surfaces have no `PreferenceKey` and cannot enter the
preference resolver or portable preference exchange.

| Surface/value | Classification and owner | Required treatment | Evidence |
|---|---|---|---|
| Dismissed-warning flags | restartable machine-local state; GUI shell | each suppression records its own prompt identity; “Restore all warnings” is an operation over state, not a bool preference | `preferences-window.html:130`; `GP_C03_PROJECT_SEED_AND_CONTEXT_DECISION_PACKET.md:726-793` |
| Active grid selection, crosshair choice, layer visibility map, terminal theme cycle | Session/restartable state | may contribute through approved Session scope; no deferred opening-default proposal converts live state | `GP_C01_INTERNAL_AUTHORITY_AUDIT.md:133-155,228-254`; `preferences-window.html:126,168,195` |
| Explicit check profile, route objective, output job, export destination, publish-set name | operation input | pre-fill descriptors may suggest; invocation must retain and audit the explicit value | `preferences-window.html:154,182-189,258-260`; `GP_C03_PROJECT_SEED_AND_CONTEXT_DECISION_PACKET.md:726-793` |
| Dimensioning units, drafting standard, sheet numbering/document conventions, effective provisional watermark | Project/document authority | shown read-only; mutation goes through Project/document authority. Only a separately defined future `ProjectPolicySeed` may initialize a new Project's watermark presentation | `preferences-window.html:147-149,184-186`; `PRODUCT_MECHANICS_035_ADOPTED_DRAFTING_STANDARD_AUTHORITY.md:18-51` |
| Guided setup completed/dismissed/never-run | machine-local onboarding state | replay/reset operation walks real rows and applies no setting | `preferences-window.html:252-254`; `guided-setup-study.html:181-254` |
| Recent Projects, missing-path status, last open documents/session | restartable state/engine query | Start page reports engine truth; no preference contribution and no startup network load | `start-page-study.html:184-249`; `preferences-window.html:239` |
| Store location, generation/head, backups, corrupt/recovery/migration/restore status | repository state/query | Manage preferences operations obey GP-C04; they do not resolve as settings | `preferences-window.html:282-299`; `preference-store-states-study.html:170-323` |
| Reset/export/import/remove unknown/migrate now/backup/restore | store operations | protected preview/confirm/result transactions; never descriptors | `preferences-window.html:285-299`; `GP_C04_STORAGE_MIGRATION_EXCHANGE_RECOVERY_CONTRACT.md:257-438` |
| Unknown keys and retired alias records | opaque repository records | inactive until exact descriptor registration; preserve bytes; alias migrates to one live identity | `preferences-window.html:288-297`; `GP_C03_PROJECT_SEED_AND_CONTEXT_DECISION_PACKET.md:1264-1345` |
| Organization authority release | registered machine-local authority state | user-held/revocable grant consumed before resolver eligibility; not an ordinary preference and not portable | `preferences-window.html:304`; `GP_C03_TYPED_AUTHORITY_DECISION_PACKET.md:719-787` |
| Pending organization requests | authority-provider query | visible/inert requests; review/dismiss operations, no prompt and no descriptor | `preferences-window.html:306`; `GP_C03_TYPED_AUTHORITY_DECISION_PACKET.md:719-787` |
| Managed policy package status | authority-provider query | read-only provider/generation/freshness/provenance | `preferences-window.html:308`; `GP_C03_TYPED_AUTHORITY_DECISION_PACKET.md:719-787` |
| Active Project policy/revision mirrors and all planned mirror detail | Project-authority query | searchable/read-only doorway; never machine preference, export, reset, or store operation | `preferences-window.html:313-317`; `GP_C05_INTERACTION_AND_VISUAL_CONTRACT.md:181-190` |
| Terminal executable/argv/cwd/environment supplied to one launch | operation input | explicit launch request wins for that invocation after Capability validation; never silently captured as a profile | `GP_C01_INTERNAL_AUTHORITY_AUDIT.md:180-204` |
| Environment and CLI compatibility inputs | external operation/process inputs | descriptor migration requires an explicit named adapter and provenance; no ambient precedence | `GP_C01_INTERNAL_AUTHORITY_AUDIT.md:206-240`; `GP_C03_TYPED_AUTHORITY_DECISION_PACKET.md:392-467` |

## 4. Explicitly deferred candidates

Every item in this section is **not active in V1**. Unless a row explicitly
names a reserved former key, it has no stable key allocated, schema/default
ratified, persistence or management eligibility, or migration identity. Every
item has implementation disposition `defer`. A later governed step must
prove the owning behavior, classify it, render any visible behavior through the
Claude lane, and amend this catalog before registration. Grouping here does not
authorize one aggregate descriptor.

| Proposed owner/section | Deferred candidates | Exact source |
|---|---|---|
| Appearance | Text render-intent backend override; anti-aliasing/render quality; icon theme/scale/display scaling; application language; hardware graphics acceleration; large-design performance mode; color-vision deficiency assistance | `preferences-window.html:110-111` |
| Workspace/input | Grid-size preset set/selection (clay); persistent crosshair opening default (clay); mouse/wheel/touchpad mapping; camera/pointer behavior; selection modifiers; cross-probe behavior; hover/tooltip/HUD assistance; numeric spin increments; double-click behavior; 3D input tuning; resource-usage warnings; undo-history/session-journal memory bound | `preferences-window.html:116-127,132-133` |
| Schematic | Wire-editing defaults; placement/annotation defaults; default schematic primitive properties; editing-focus emphasis | `preferences-window.html:149-151` |
| PCB/board | Opening-layer-visibility default (clay); auto-via on layer change; interactive-routing defaults; routing-objective default; rotation/flip direction; track/arc editing mode; default board primitives; polygon-repour automation; net-name/clearance display; displayed origin/axes; zone display mode | `preferences-window.html:168-171` |
| Publish/document | Publishing destinations | `preferences-window.html:186-188`; historical seed `GP_DRAFT_SETTINGS_CATALOG_SEED.md:123` |
| Rules/checks | Live DRC while editing; DRC violation-overlay style | `preferences-window.html:191` |
| Terminal | Graphics policy; process-close/kill confirmation; copy-format policy; font fallback chain | `preferences-window.html:215-216` |
| Library | Pools/search order; IPC footprint naming/basis default; field-name templates; supplier data/currency; browser zoom; library/model path variables | `preferences-window.html:218-219` |
| Symbol editor | New-symbol defaults; pin length/spacing; graphics line width | `preferences-window.html:221-226` |
| Footprint editor | Pad defaults; IPC naming basis/density; silk/courtyard widths; reference/value text; preview annotations | `preferences-window.html:229-236` |
| Files/projects | `ProjectPolicySeed` candidate for `AdoptedDraftingStandard` (one of fourteen reserved seed-bearing rows; schema unspecified); multi-instance locking; helper applications; update checks; cache maintenance; 3D model search paths; update channel | `preferences-window.html:255-256` |
| Revision | PM-038 recovery of reserved keys `datum.revision.visibility`, `datum.revision.profile_seed`, `datum.revision.build_presentation_seed`, and `datum.revision.prototype_transition_seed`; one ProjectPolicySeed candidate for new-Project provisional-watermark presentation; Git/offline-exchange seed; local-history/session-journal retention; machine VCS integration; user identity for provenance. The reserved keys have no active descriptor/default/management authority and unmanaged Projects show no Revision chrome. | `PRODUCT_MECHANICS_038_REVISION_RECOVERY_AND_PRODUCT_BASELINE.md`; `preferences-window.html:264-272` |
| Agents | Agent authority level including unattended, unattended-tool allowlist, persistent agent Project configuration, scripting/agent API access, engine-daemon/MCP endpoint configuration; all await dedicated authority/security review | `preferences-window.html:274-279` |
| Organization/network | Organization-managed pins/default packaging; network-access toggles; telemetry/crash-reporting opt-in; proxy configuration | `preferences-window.html:310-311` |

The historical intake was 60 current plus 59 planned. PM-036 subsequently moved
“Editor appearance themes” from planned to the active schematic-theme
descriptor, so the prototype comparison became 61 current plus 58 planned with
the same total of 119; neither banner is a registry count. “Title-block scale
fraction (per-firm)” became a typed publish-space seed. Historical Project-mirror
candidates remain negative classifications, not deferred descriptors. Later
editor and completeness-critic rows are post-intake deltas and are accounted for
by their explicit active, deferred, or negative disposition here.

## 5. Migration, retirement, and implementation boundary

1. The only implemented persistent preference today is the legacy Console
   duration field. Its V1 migration targets
   `datum.console.feedback_duration`; successful migration leaves one identity.
2. The terminal notification environment input is not silently imported. A
   bounded compatibility adapter may create a typed contribution only when its
   source, lifetime, and removal rule are explicit and audited.
3. `terminal.legacy_bell_mode` is currently unknown/newer-version evidence. It
   remains byte-faithful and inactive until a future migration declares its
   schema and maps it without inventing a value.
4. Every other active descriptor is `implement new`. Absence never masquerades
   as a contribution: the resolver may expose a descriptor default, but
   implementation must not manufacture stored defaults during first
   run, import, recovery, or migration.
5. Renaming a GUI label never renames a key. Retired and alternate names remain
   searchable vocabulary under GP-C05 even when they are not storage aliases.
6. Capability values, authority releases, credentials, expanded paths, Project
   policy, design data, repository generations, onboarding state, and
   restartable state are excluded from portable preference exchange.
7. No migration repairs an unreadable store in place, substitutes a value the
   user did not choose, drops unknown data, or blocks authoring.

## 6. Ratified conformance checks

GP-C06 must adversarially verify at least:

- key uniqueness, owner presence, schema/default validity, source eligibility,
  Q3/Q4 management bounds, and complete provenance for every active descriptor;
- all seed destinations against Q5 copy-once/receipt law and Project authority;
- live/restart/next-open behavior against actual consumers;
- personal accessibility carve-outs, each justified by a named descriptor, and
  keyboard/non-color/reduced-motion presentation;
- Capability refusal from portable and Project sources;
- exact preservation of unknowns and one-live-identity alias migration;
- exclusion of unsettled clay rows and unreviewed agent-authority descriptors;
- complete accounting of the 119 historical candidates and later prototype
  deltas without treating prototype counts as descriptor authority.

<!-- EVIDENCE:GLOBAL-PREFERENCES-SPEC:GP-C08-V1-DESCRIPTOR-CATALOG -->
