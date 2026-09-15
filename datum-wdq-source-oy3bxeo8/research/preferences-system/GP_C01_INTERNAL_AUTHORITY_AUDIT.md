# GP-C01 Internal Preferences Authority Audit

> **Status:** factual baseline prepared for `GP-C01A` owner review; no future
> architecture, external research, implementation, dependency, or standards
> conformance is authorized by this document.
>
> **Tracker:** `dat-global-preferences-engine-qcv`
> **Frontier step:** `GLOBAL-PREFERENCES-SPEC / GP-C01`

## 1. Audit question and boundary

This audit answers one factual question: **what currently controls Datum's
preference-like behavior, where does each value live, and what authority is
missing?** It classifies current product inputs rather than presuming that every
input belongs in the future Global Preferences engine.

The inventory covers:

- persistent machine-local UI preference state;
- process-launch configuration and environment inputs;
- session-only GUI and terminal state;
- Project-authored policy and data that must not be mistaken for preferences;
- per-operation choices that are inputs to a command, not lasting preferences;
- diagnostic, accessibility-platform, compatibility, and test controls;
- specification/prototype promises that have no implementation yet;
- precedence, provenance, validation, migration, recovery, and failure behavior;
- the two named Revision carry-forwards: managed visibility and first-Release
  onboarding.

The audit does **not** enumerate every ordinary domain command operand (for
example a board UUID, route endpoints, or an output path). Those are operation
parameters, not settings. It does inventory the mechanisms that can alter
long-lived behavior or look like a preference/configuration authority.

## 2. Method and reproducible evidence

The baseline was produced from the repository at commit `417d16c` plus the
live `GP-C01` claim. Evidence was collected with line-numbered source reads and
the following repository-wide searches:

```text
rg -n 'std::env::(var|var_os|vars)|env::(var|var_os|vars)' crates --glob '*.rs'
rg -n -i 'preference|preferences|managed.*revision|revision.*visibility|first[- ]release|onboarding' crates docs research specs
rg -n 'ProjectManifest|NativeProjectManifest|WorkspaceUiState|TerminalProfileArgs' crates --glob '*.rs'
```

An absence finding below means the named production paths and the full Rust
source tree were searched. It does not mean a visual study or future
specification never mentions the concept.

## 3. Findings-first verdict

1. **There is no Global Preferences engine or Preferences GUI.** The menu
   contract explicitly records `Preferences | none | NOT-BUILT`
   (`docs/gui/DATUM_GUI_MENU_BINDINGS.md:64-70`).
2. **Exactly one product UI choice persists across application launches:**
   Console auto-hide duration. It uses a Console-owned, one-field JSON file,
   not a typed global registry (`crates/gui-app/src/console_preferences.rs:1-80`).
3. **Most current user-facing choices are process-local or session-local.** Pane
   tiling, focus, zoom, filters, crosshair, terminal dock geometry, terminal
   theme/font zoom, and active terminal launch profile live in GUI state and are
   never journaled (`crates/gui-protocol/src/workspace_layout.rs:1-8,89-160`).
4. **Terminal profiles are launch arguments, not persisted preferences.** They
   currently combine presentation, resource limits, process launch, environment,
   and agent authority in one process-built catalog
   (`crates/gui-app/src/terminal_profile.rs:15-103,209-250`).
5. **Project policy is real but it is authored domain state, not a preferences
   layer.** The native manifest contains identity and shard references, not a
   global-preference snapshot or managed-policy binding
   (`crates/engine/src/substrate/project_resolver.rs:45-55`;
   `crates/engine/src/api/native_write/genesis.rs:77-89,233-242`).
6. **No implemented scope hierarchy, precedence resolver, effective-value
   provenance, organization-managed provider, Project seeding mechanism,
   preference migration, import/export, synchronization, or Preferences audit
   trail exists.**
7. **Managed Revision visibility does not exist in Rust.** The approved visual
   concept is a future presentation requirement only.
8. **First-Release onboarding and its completion/reset memory do not exist in
   Rust.** The Revision Engine itself is not implemented, and no onboarding
   state, trigger, completion record, or managed replay mechanism exists.

These are baseline facts, not a recommendation about the future architecture.

## 4. Implemented persistent preference authority

### 4.1 Console duration is the sole persistent GUI preference

| Property | Implemented fact |
|---|---|
| Owner | `gui-app` Console adapter; explicitly machine-local and never Project/journal state (`console_preferences.rs:1-4`) |
| Key | `console_duration` only (`:40-44`) |
| Type at runtime | `ConsoleFeedbackDuration`: 4 s, 6 s, 10 s, or never (`:63-79`) |
| Factory default | `SixSeconds` from `ConsoleFeedbackDuration::default` (`crates/gui-protocol/src/console_feedback.rs:47-69`) |
| User control | View-menu actions `view.console_duration.{4s,6s,10s,never}` (`crates/gui-app/src/runtime_view_actions.rs:127-145`) |
| Location precedence | `DATUM_GUI_PREFERENCES_PATH`; otherwise `$XDG_CONFIG_HOME`; otherwise `$HOME/.config`; then `datum/gui-preferences.json` (`console_preferences.rs:22-30`) |
| Value precedence | Valid file value replaces the in-memory default at Runtime creation (`crates/gui-app/src/main.rs:781-787`) |
| Write behavior | Create parent, write PID-suffixed temporary file, rename over destination (`console_preferences.rs:47-61`) |
| Failure behavior | Save failure keeps the session value and publishes a Console refusal (`runtime_view_actions.rs:158-173`) |
| Missing file | Returns no override; factory/session default remains (`console_preferences.rs:32-37`) |
| Invalid/unreadable file | Public loader discards the error through `.ok().flatten()` and silently retains the default (`console_preferences.rs:12-15`) |
| Schema handling | Writer emits the string `datum_gui_preferences_v1`, but loader never reads or validates `schema` (`console_preferences.rs:38-45,53-56`) |
| Unknown fields | Load ignores them; the next write replaces the whole document and discards them (`:38-60`) |
| Migration | None |
| Recovery | Rename-level replacement only; no backup selection, journal, fsync protocol, or corrupt-file recovery |
| Concurrency | No lock, compare-and-swap, generation, or merge; temp name is process-scoped only |
| Permissions | Inherits ordinary `create_dir_all`/`write` defaults; no explicit private-mode enforcement |
| Provenance | No query explaining default versus file versus path environment override |

The GUI design spec's statement that Console duration “persists atomically as
machine-local UI preference state” accurately describes the intended temp-write
and rename boundary (`docs/gui/DATUM_GUI_DESIGN_SPEC.md:380-387`), but must not be
read as evidence for a general Preferences authority or complete crash recovery.

### 4.2 Current effective precedence is narrow and implicit

For Console duration only, current behavior is:

```text
compiled SixSeconds default
  < valid value in the selected preference file

preference-file path:
DATUM_GUI_PREFERENCES_PATH
  > XDG_CONFIG_HOME/datum/gui-preferences.json
  > HOME/.config/datum/gui-preferences.json
```

There is no organization scope, Project scope, command-line value override,
session override record, contextual rule, lock, provenance object, or effective-
value query. A GUI mutation immediately changes session state before persistence
is attempted, so a failed write deliberately produces a session-only divergence.

## 5. Session-only GUI controls

`WorkspaceUiState` is an in-memory consumer-state bag. Its module explicitly
excludes the entire bag from design operations and the journal
(`workspace_layout.rs:1-8`). Current preference-like values are:

| Surface | Values/default | Lifetime and evidence |
|---|---|---|
| Crosshair | full viewport (default), local, none | Session-only; set/cycle in `runtime_view_actions.rs:187-215`; type/default at `workspace_layout.rs:16-30` |
| Pane tree | Board+Schematic vertical 0.5 default; recursive split ratios clamp 0.1–0.9 | Session-only layout/focus/zoom (`workspace_layout.rs:300-360`) |
| Pane presets/content | single, Board+Schematic; Board or Schematic in focused pane | View actions only (`runtime_view_actions.rs:50-116`) |
| Dock | closed by default; 220 logical-pixel stored height; transient maximize | Defaults at `workspace_layout.rs:140-160`; runtime resize/maximize is in `runtime_terminal_dock.rs:349-387,505-506` |
| Scene filters | authored, proposed, unrouted, dim-unrelated, active layer, per-layer visibility | Stored only in `WorkspaceFilterState` (`workspace_layout.rs:88-96`) |
| Menus/hover/cursor | active menu, marking menu, hover targets, cursor position | Explicit transient presentation state (`workspace_layout.rs:99-133`) |
| Terminal theme | profile-seeded, then cycles in memory | `runtime_terminal_theme.rs:1-11` |
| Terminal font zoom | profile-seeded; 60%–200%, 10% steps; reset to 100% | `runtime_terminal_font.rs:9-40` |
| Terminal selected launch profile | default/login/custom catalog; cycles for new terminals | `runtime_terminal_profile.rs:1-17` |
| Terminal tabs/order/sessions | live terminal lane state | Session/PTY state, never Project design state (`docs/gui/DATUM_NATIVE_TERMINAL_SPEC.md:36-44`) |
| Console history visibility/history | bounded session projection | `WorkspaceUiState.console` and `.console_journal` (`workspace_layout.rs:128-132`) |

No load/save path was found for the workspace layout, pane ratios, last focused
pane, dock height, crosshair, filters, terminal theme/font zoom, or selected
terminal profile. “Persistent” in several GUI documents often means “continuously
visible chrome,” not “persisted across restarts”; those meanings must remain
separate in the future vocabulary.

## 6. Process-launch configuration

### 6.1 GUI application arguments

`GuiArgs` is reconstructed on every process launch
(`crates/gui-app/src/app_bootstrap.rs:18-98`). Its inputs divide as follows:

- **Product opening/context inputs:** `--board`, `--schematic`, `--artifact`,
  `--project-root`, `--select`, net/from/to anchors, and artifact `--profile`.
  These select what the process opens; they are not stored user preferences.
- **Terminal launch configuration:** the flattened terminal profile arguments
  described below.
- **Capture/test inputs:** known-good demo, visual-test mode, initial layout,
  initial focus, open menu, window size, screenshot output, scale override, and
  smoke-test modes. Source comments explicitly identify layout/focus as capture
  affordances and consumer state (`app_bootstrap.rs:55-97,129-177`).

No process argument is represented in a unified configuration provenance model.
Clap supplies the shown defaults, and the launch builder writes selected terminal
profile values into new session state (`app_bootstrap.rs:333-348`).

### 6.2 Terminal launch profile catalog

`TerminalProfileArgs` currently exposes:

| Class | Inputs | Defaults/validation |
|---|---|---|
| Selection/identity | selected profile; custom name | `default`; custom name `custom`; only default/login/configured custom may resolve (`terminal_profile.rs:15-25,225-239`) |
| Process | executable, exact argv, cwd | `$SHELL` then `/bin/sh`; cwd `active`; relative paths resolve under Project (`:23-37,175-205,272-276`) |
| Environment | repeated set and remove | inherited environment modified by validated keys (`:29-34,332-366`) |
| Appearance | theme, font scale, cursor, visual bell | Datum Dark; 100%; blinking block; visual bell (`:38-56,105-123,277-330`) |
| Resources | scrollback lines and MiB | production core limits; bounded to 100,000 lines and 64 MiB (`:44-49,296-315`) |
| Agent authority | inspect/propose/apply-approved/unattended plus exact unattended tools | `propose`; validated at catalog construction (`:57-62,209-224`) |

The catalog is assembled in memory each launch. Default and login profiles are
hard-coded; at most one custom profile is synthesized from arguments
(`terminal_profile.rs:209-250,252-360`). Cycling changes only the current Runtime
and future terminal launches (`runtime_terminal_profile.rs:5-17`). There is no
persistent profile store, schema, migration, organization pin, Project seed,
import/export, or effective-value explanation.

This surface also mixes unlike authorities. Theme/font/cursor/bell are
presentation choices; executable/argv/cwd/environment are launch templates;
scrollback values are resource policy; agent authority and unattended-tool
allowlists are security/automation policy. GP-C03 must classify them before any
of them can become “preferences”; GP-C01 does not collapse them.

## 7. Environment and ambient configuration inventory

### 7.1 Product behavior inputs

| Input | Current behavior | Classification |
|---|---|---|
| `DATUM_GUI_PREFERENCES_PATH` | Overrides Console preference-file path (`console_preferences.rs:10,22-30`) | machine/process bootstrap override |
| `XDG_CONFIG_HOME`, `HOME` | Select fallback Console preference location (`:26-29`) | operating-system location input |
| `SHELL` | Supplies terminal executable, fallback `/bin/sh` (`terminal_profile.rs:175-184`) | ambient launch input |
| `DATUM_TERMINAL_NOTIFICATIONS` | `off`, `always`, otherwise/default `unfocused` (`runtime_terminal_notifications.rs:34-44`) | process-local presentation policy; unknown silently becomes `unfocused` |
| `DATUM_DISCOVERY`, `DATUM_TERMINAL_CONTEXT` | CLI context path precedence after explicit `--path`, before Project-derived fallback (`crates/cli/src/context/mod.rs:218-243`) | discovery/runtime context, not a user appearance preference |
| `DATUM_AGENT_DISCOVERY`, `PATH` | Resolve agent discovery/executable (`crates/cli/src/commands/agent.rs:133-158`) | tool discovery |
| `DATUM_COMMIT_SOURCE` | Selects typed CLI/tool/assistant provenance; invalid values refuse (`crates/cli/src/commands/support.rs:18-35`) | mutation provenance, not preference |
| `DATUM_AGENT_PROVENANCE` | Parses and validates UTF-8 JSON agent provenance (`crates/engine/src/substrate/commit.rs:207-223`) | mutation provenance, not preference |
| MCP server override | Overrides MCP executable path (`crates/cli/src/commands/mcp.rs:60-75`) | adapter/bootstrap configuration |
| `EDA_CLI_BIN` | Selects CLI binary used by GUI protocol helper (`crates/gui-protocol/src/lib.rs:4416-4424`) | executable discovery |
| `AT_SPI_BUS_ADDRESS`, `DBUS_SESSION_BUS_ADDRESS` | Select accessibility bus (`crates/gui-app/src/terminal_accessibility_platform/connection.rs:50-69`) | platform integration |
| `LC_ALL`, `LC_CTYPE`, `LANG` | Select terminal accessibility locale (`terminal_accessibility_platform/atspi.rs:580-590`) | platform locale |

No common precedence registry or provenance query spans these inputs. Each
module implements its own parsing, default, invalid-value, and fallback rules.

### 7.2 Diagnostic, proof, compatibility, and test-only inputs

`DATUM_GUI_LOG`, `DATUM_GUI_VERBOSE_LOG`, `DATUM_TRACE_TIMING`,
`DATUM_TRACE_GRAPHICS`, `DATUM_TRACE_CLICKS`, and `DATUM_TRACE_IMPORT_TEXT`
control diagnostics/tracing. `UPDATE_GOLDENS`, `DATUM_DTC_P23_SCREENSHOT_OUT`,
`DATUM_LOW_FD_REPORT`, `DATUM_RUN_EXTERNAL_DOA2526_TESTS`, the `DATUM_P06_*`,
`DATUM_P28_*`, `DATUM_AGENT_CLI_PROOF_BIN`, and DOA2526 fixture paths are
proof/compatibility inputs. They are not candidates for automatic migration
into a user Preferences UI merely because they are environment variables.

Manufacturing and artifact planners also capture the ambient environment in
run/evidence records. That is reproducibility evidence, not preference
authority (`crates/cli/src/commands/manufacturing/manufacturing.rs:684` and
the artifact/gerber run planners).

## 8. Project policy and per-operation choices

### 8.1 Native Project contains no Global Preferences binding

The resolved native manifest currently carries optional schema version, Project
UUID/name, pool references, and schematic/board/rules shard paths
(`project_resolver.rs:45-55`). Genesis writes schema version 1 and those roots,
plus an empty forward-annotation-review map
(`native_write/genesis.rs:77-89,233-242`). There is no field for:

- source Global Preferences profile or seed receipt;
- copied Project policy snapshot;
- organization-policy binding;
- effective preference values or provenance;
- preference schema/provider identity;
- Revision policy, managed Revision visibility, or onboarding completion.

### 8.2 Existing Project policy is domain authority

Rules shards, net classes, pool priority, waivers/deviations, manufacturing
plans, output jobs, artifact definitions, variants, and forward-annotation
review are authored Project facts. They resolve and mutate through native shard
and journal mechanisms. The automated-write proposal requirement is hard-coded
by operation family, not selected through preferences
(`crates/engine/src/substrate/proposal_policy.rs:1-79,81-147`).

These facts must not be silently overridden by machine-local presentation
preferences. Future new-Project seeding can copy an approved policy into Project
authority, but GP-C01 found no such copying mechanism today.

### 8.3 Invocation-local choices are not persisted preferences

Route proposal profile, import merge behavior, check/run target or profile,
artifact/output selection, formatting mode, and similar CLI arguments select one
operation or report. They have typed validation in their owning command/domain,
but no persistent user scope, managed scope, or cross-command effective-value
resolution. GP-C03 may decide whether any deserves a remembered default; GP-C01
does not reclassify them.

## 9. Specification and implementation contradictions

| ID | Baseline finding |
|---|---|
| `GP-GAP-01` | The product menu promises a Preferences entry, while the binding ledger correctly says no mechanism exists (`DATUM_GUI_PRODUCT_SPEC.md:120-129`; `DATUM_GUI_MENU_BINDINGS.md:64-70`). |
| `GP-GAP-02` | The filename/schema label `gui-preferences` suggests a general store, but its only supported key is Console duration and its schema field is not validated. |
| `GP-GAP-03` | Console persistence is called atomic, but recovery is limited to temp-write/rename; no durability sync, backup, corrupt-file recovery, lock, or generation exists. |
| `GP-GAP-04` | Unknown Console fields survive reads but are erased on the next write, contradicting the planned requirement to preserve unknown data through version changes (`GLOBAL_PREFERENCES_ENGINE_RESEARCH.md:10-15`). |
| `GP-GAP-05` | GUI guidance names numerous future “Preferences” (appearance, airwires, pads, copper opacity, auto-via, reduced motion), but no registry, storage, or Preferences UI implements them. These are requirements/candidates, not current settings. |
| `GP-GAP-06` | Workspace and terminal choices are described as user selections but reset at process start unless injected by launch arguments; no per-user or per-Project layout memory exists. |
| `GP-GAP-07` | Terminal profile arguments combine presentation, execution, resource, and agent-security policy without a typed cross-scope authority or provenance display. |
| `GP-GAP-08` | Environment inputs implement independent, inconsistent invalid-value behavior: some refuse, some silently default, some only toggle on presence. |
| `GP-GAP-09` | Native Project schema has no explicit policy-seeding receipt or separation between copied Project authority and continuing global defaults. |
| `GP-GAP-10` | No import/export, synchronization, conflict, migration, downgrade, unavailable-provider, reset, or audit mechanism exists for preferences. |
| `GP-GAP-11` | No accessible Preferences discovery/search/editing surface or effective-value/managed-state explanation exists. |

## 10. Named Revision carry-forward proofs

### 10.1 Managed Revision visibility: absent

The owner requirement is recorded at
`research/preferences-system/GLOBAL_PREFERENCES_ENGINE_RESEARCH.md:23-35`.
The approved visual study shows the future `Hide revision system` control and
explicitly defers its mechanism to Global Preferences
(`docs/gui/prototypes/revision-ux-shell-study.html:236-271`).

A production Rust search for managed Revision policy, Revision visibility, and
the literal control name returns no matches. There is currently:

- no Revision visibility setting;
- no user hide value;
- no organization-managed provider or “pin on” constraint;
- no lock/refusal behavior;
- no effective-value provenance or reason;
- no stored managed-policy identity.

The permanent expanded Revision Navigator groups are approved visual/spec
direction, not implemented GUI state. This absence is expected because Product
Revision implementation is sequenced after Global Preferences policy resolution.

### 10.2 First-Release onboarding/completion state: absent

The owner requirement is recorded at
`GLOBAL_PREFERENCES_ENGINE_RESEARCH.md:37-53`. No production Rust match exists
for first-Release onboarding, onboarding completion, tutorial reset, or managed
replay. There is currently no:

- trigger distinguishing a user's first Release from later Releases;
- persisted completion/dismissal state;
- user, machine, organization, or Project scope for that state;
- reset or replay operation;
- accessible onboarding presentation;
- integration with the future arm-then-confirm Release boundary.

Prototype empty-state wording (“issue your first release to begin”) is teaching
copy, not onboarding state (`revision-ux-shell-study.html:170-194`). Product
Revision research explicitly states that no general Preferences storage,
precedence, migration, or GUI exists
(`research/documentation-system/PRODUCT_REVISION_ENGINE_RESEARCH.md:309-317`).

## 11. Missing authority matrix for GP-C02–GP-C05

| Capability | Current state | Earliest planned resolution |
|---|---|---|
| Typed preference key/registry/value schema | absent | GP-C03 |
| User/machine, managed organization, Project seed, session, contextual scopes | absent as one model | GP-C03 |
| Deterministic precedence and effective-value provenance | Console-only implicit fallback | GP-C03 |
| Constraints, locks, refusal, managed reasons | absent | GP-C03 / GP-C05 |
| Project-policy seed/copy receipt and post-copy independence | absent | GP-C03 / GP-C04 |
| Canonical local store | one-field Console file only | GP-C04 |
| Atomic durability and recovery | rename-only Console write | GP-C04 |
| Schema/value migration and downgrade behavior | absent | GP-C04 |
| Unknown key/provider preservation | absent; Console rewrite discards | GP-C04 |
| Import/export and sync/conflict behavior | absent | GP-C04 |
| Accessible Preferences UI/search/provenance/reset | menu placeholder only | GP-C05 |
| Managed Revision visibility | requirement/prototype only | GP-C03 / GP-C05 |
| First-Release onboarding state and presentation | requirement only | GP-C03 / GP-C05 |

## 12. Factual baseline proposed for owner disposition

<!-- EVIDENCE:GLOBAL-PREFERENCES-SPEC:GP-C01-INTERNAL-AUDIT -->

The GP-C01 baseline is:

> Datum does not currently have a Global Preferences engine. It has one narrow,
> Console-owned machine-local preference file; independent process/environment
> configuration paths; extensive session-only GUI state; and authoritative
> Project/domain policy that must remain distinct from preferences. No unified
> typed scope, precedence, provenance, managed policy, Project seeding,
> migration, recovery, exchange, synchronization, accessibility surface,
> managed Revision visibility, or first-Release onboarding state exists.

This statement is deliberately descriptive. `GP-C01A` asks the owner to approve
or revise these facts before external research or architecture decisions begin.
