# Product Mechanics Decision 005: Embedded Terminal

Status: draft hypothesis + implementation mechanism woven 2026-06-19.
Date: 2026-06-19

Driven by:
- `docs/DATUM_PRODUCT_MECHANICS.md`
- `docs/audits/scope-integration/DATUM_PRODUCT_MECHANICS_REVIEW_AGENDA.md`
- `docs/decisions/PRODUCT_MECHANICS_000_UNIFIED_DESIGN_MODEL.md`
- `docs/decisions/PRODUCT_MECHANICS_000D_STORAGE_AND_VERSIONING_MODEL.md`
- `docs/decisions/PRODUCT_MECHANICS_000F_LIVE_PRODUCTION_PROOF_MODEL.md`
- `docs/decisions/PRODUCT_MECHANICS_001_CANONICAL_EDIT_MODEL.md`
- `docs/decisions/PRODUCT_MECHANICS_004_AI_TOOLING_CONTRACT.md`

## Decision Scope

Define the product quality bar and implementation boundaries for Datum's
embedded terminal.

This decision distinguishes a real PTY-backed terminal emulator from a fake
command pane, command palette, agent proxy, Datum-only console, or hidden
mutation bridge.

## Product Intent

Datum should include a real embedded terminal because professional EDA work
often involves normal shell workflows: version control, scripts, builds,
linters, manufacturing file inspection, vendor tooling, project automation,
and optional AI agents.

The terminal must support user choice. A user should be able to launch
`codex`, `claude`, `aider`, shells, scripts, compilers, Datum CLI commands, or
other installed tools from inside Datum.

The terminal is necessary for an AI-friendly workflow, but it is not the full
AI architecture. AI-native integration comes from Datum's EDA-native CLI/MCP
tools, stable IDs, structured context, proposals, diffs, checks, artifacts,
transactions, and provenance. A terminal beside an unrelated EDA app is not
enough.

## Decision

Datum shall implement the embedded terminal as a normal PTY-backed terminal
surface plus stable Datum context injection.

The terminal may launch arbitrary user commands, but it grants no private
design mutation powers. Terminal-launched tools can inspect and propose changes
only through the same Datum CLI/MCP/API contract used by assistants, scripts,
manual tooling, importers, checkers, and routers. Any accepted design mutation
must become an `OperationBatch`, pass validation, call `commit()`, append a
`TransactionRecord` to the journal, and record terminal/process provenance.

The terminal is a process/session surface. It is not a project authority, not
a second undo stack, not a shell-text-to-edit interpreter, and not a bypass
around the proposal/transaction model.

## User-Visible Behavior

The embedded terminal should behave like a normal Linux terminal emulator:
- launch the user's real shell, such as `$SHELL`
- allocate a real PTY for each terminal session
- run arbitrary shell commands, not just Datum commands
- preserve `cwd`, shell state, environment, running processes, scrollback, and
  terminal title per session while Datum is running
- support ANSI/VT behavior, colors, cursor movement, alternate screen,
  interactive programs, resize events, keyboard input, copy, paste, selection,
  search, and scrollback
- support long-running and interactive processes
- support multiple terminal tabs or sessions
- start in the active project root when launched from a project
- expose useful Datum context through environment variables and a discovery
  command without constraining the shell

Terminal tabs are shell sessions. They are not assistant conversations unless a
user explicitly starts an assistant program inside the shell.

## Terminal Session Model

### TerminalSession

`TerminalSession` is one PTY-backed process tree.

Required fields:
- `terminal_session_id`
- `project_id` when launched from a project
- `project_root_at_launch`
- `pty_id`
- child process PID/process group where available
- launched executable and argv
- initial `cwd`
- current `cwd` best-effort tracking where the platform supports it
- environment snapshot with Datum additions marked separately
- terminal size
- title
- scrollback buffer reference
- lifecycle state: `Running`, `Exited`, `Detached`, `Restartable`,
  `Terminating`, or `Closed`
- `DatumContextEnvelope` snapshot ID

The PTY process owns shell state. Datum must not attempt to reconstruct shell
aliases, functions, prompt state, job control, or process internals as product
state.

### TerminalTab

`TerminalTab` is a UI binding to one `TerminalSession`.

Tabs can attach, detach, rename, close, and restart sessions according to the
session lifecycle. Closing a tab must make clear whether the underlying process
continues, receives SIGHUP/termination, or is already exited.

### SessionLifecycle

Minimum lifecycle operations:
- create session from project context
- attach UI tab to running session
- detach UI tab without killing process where supported
- rename tab/session title
- resize PTY
- close tab
- terminate process tree
- restart a new session from the original launch template

First cut persistence is process-local: sessions persist across tab switches
and project UI navigation inside one running Datum process. Persisting live
PTY sessions across Datum application restart is not required; restoring
metadata, recent commands, and scrollback snapshots is an owner question.

### cwd And Environment

When launched from a project, the initial `cwd` is the active project root.
When launched from a selected artifact/output context, Datum may offer a
secondary launch action into that directory, but project-root launch remains
the default.

The shell receives the user's inherited environment plus Datum-specific
variables. Datum variables must not override user variables except by explicit
namespaced keys.

Required first-slice variables:
- `DATUM_PROJECT_ROOT`
- `DATUM_PROJECT_ID`
- `DATUM_MODEL_REVISION`
- `DATUM_CONTEXT_ID`
- `DATUM_SESSION_ID`
- `DATUM_DISCOVERY`
- `DATUM_CLI`

Optional variables:
- `DATUM_ACTIVE_PROJECTION_ID`
- `DATUM_SELECTION_IDS`
- `DATUM_ARTIFACT_DIR`
- `DATUM_MCP_ENDPOINT`

`DATUM_MODEL_REVISION` and selection variables are snapshots. Terminal tools
must refresh context before creating or applying proposals if the model
revision has moved.

### Stable Context Injection

Datum injects context through two stable mechanisms:
- environment variables for bootstrapping
- `datum-eda context get|refresh --session $DATUM_SESSION_ID` for current
  structured context

CLI naming note: `datum-eda` is the canonical CLI executable. `eda` and bare
`datum` command examples are legacy/noncanonical; `datum.*` remains reserved
for MCP tool family names only.

If a local MCP/session socket is enabled, `DATUM_DISCOVERY` points to the
discovery record that tells tools how to connect. The socket is an access path
to the same CLI/MCP contract classes defined in Decision 004, not a privileged
mutation channel.

Context refresh returns a `DatumContextEnvelope` containing project root,
project ID, model revision, accepted transaction tip, active projection,
selection, visible findings/artifacts, and available capabilities. The
envelope is a read-only snapshot of the engine-owned resolved `DesignModel`
assembled by `ProjectResolver` at a specific `model_revision`; the
terminal/CLI/MCP path only consumes that resolved model and never assembles
or authors it. It does not expose source shard bytes as authority.

## Provenance Boundaries

Datum records terminal provenance only when terminal-launched work crosses a
Datum boundary:
- a CLI/MCP query may be logged as session telemetry or local session history
  depending on product policy
- a check run that becomes project state records command/session provenance,
  and lands as a revision-keyed `CheckRun`/`CheckFinding` per Decision 009
  recomputed and invalidated by `model_revision`, never persisted as authored
  authority
- an artifact generated by a terminal-launched command is a full Artifact per
  the Decision 011/012 schema (authoritative): artifact id, type, source
  `model_revision`, board/panel context, variant, manufacturing plan, output
  job, generator version, validation state, and content hash/path. The command
  and terminal session are recorded as additional provenance; they do not
  replace any required Artifact metadata field
- a proposal created by a terminal-launched tool records terminal session,
  executable/argv, actor identity, context revision, and affected objects
- an accepted proposal or allowed direct commit records the proposal or command
  as the transaction provenance and acceptance path

Datum must not record arbitrary shell input as design history merely because a
terminal exists. Project history records design-significant outcomes:
proposals, transactions, artifacts, checks, and imported evidence.

## Manual Workflow Requirements

The terminal must improve manual workflows independently of AI.

Users must be able to:
- run `git status`, `git diff`, `git commit`, and other normal VCS commands
- run Datum CLI commands against the active project
- run scripts and external tools from the project directory
- inspect generated manufacturing artifacts
- use shell navigation, environment variables, aliases, shell history, and
  interactive TUI programs
- copy command output into reports, issues, or notes
- manage multiple sessions without losing working directory or process state
  during the running Datum process

The terminal must not become the only path to core EDA capability. Users still
need manual schematic, PCB, library, rules, checks, and manufacturing tools.

## Optional AI And Tooling Behavior

Optional AI tooling may use the terminal exactly as a user would:
- launch `codex`, `claude`, `aider`, or other installed tools
- run Datum CLI commands
- connect to Datum MCP/session services if configured
- inspect files and generated artifacts
- run checks and scripts
- create proposals or call proposal APIs

Datum may make terminal-launched agents more useful by exposing stable project
context, CLI/MCP discovery, output artifact directories, and command examples.

The terminal itself must remain a normal shell surface. It must not secretly
translate terminal text into Datum edits or grant terminal processes private
mutation powers. Any design mutation from a terminal-launched tool must still
flow through the canonical operation, proposal, transaction, diff, check, and
provenance model.

## Standards And Compliance Impact

The terminal can launch tools that inspect, validate, generate, or package
standards-sensitive artifacts. Datum must still preserve compliance
boundaries:
- terminal-generated artifacts carry the full ratified Artifact metadata set
  defined by Decision 011/012 (artifact id, type, source `model_revision`,
  board/panel context, variant, manufacturing plan, output job, generator
  version, validation state, content hash/path); command and environment
  context are recorded as additional provenance, not as a substitute for any
  required field
- terminal-launched check or generation commands should emit
  machine-readable findings or artifact manifests when they affect Datum state
- standards fixes proposed by terminal-launched tools should become proposals,
  not silent edits
- generated manufacturing outputs should remain tied to the model revision,
  variant, output job, manufacturing plan, and validation state

The terminal must not be a loophole for untracked standards waivers, imported
geometry repair, or manufacturing output mutation.

## Private Mutation Ban

Terminal-launched commands may write ordinary files that the user has
permission to write, such as notes, scripts, logs, build outputs, and exported
artifacts. They may not create design-state authority by editing Datum source
shards, derived caches, or the transaction journal outside `commit()`.

Datum must detect and surface dirty project-source changes that appear outside
the journal when possible. Such changes are treated as external file edits to
import, reconcile, or reject; they are not silently accepted as committed
Datum transactions.

## Explicit Non-Goals

This decision does not define:
- a custom shell
- a Datum-only command language
- a fake prompt with limited commands
- a required AI agent
- a package manager or tool installer
- remote terminal sharing
- cloud execution
- full terminal multiplexing parity with `tmux`
- direct design mutation by parsing shell text

## First Proof Slice

The first proof slice should demonstrate a project-scoped real terminal:
- create one PTY-backed terminal tab from a Datum project
- launch the user's configured shell in the project root
- run normal commands such as `pwd`, `ls`, and a Datum CLI query
- preserve `cwd` after `cd` while the session is running
- support interactive input and long-running commands
- handle ANSI color output and resize
- provide copy/paste and scrollback
- launch an installed agent command when present without special integration
- expose `DATUM_PROJECT_ROOT`, `DATUM_MODEL_REVISION`, `DATUM_SESSION_ID`,
  `DATUM_DISCOVERY`, and `DATUM_CLI`
- create a proposal from a terminal-launched Datum CLI command and apply it
  through the canonical proposal/transaction path

The proof does not need complete terminal-emulator parity, but it must prove
the architecture is a real PTY terminal rather than a text command widget and
that terminal-launched mutation cannot bypass `commit()` and the journal.

## Open Owner Questions

1. Which terminal emulation library or platform abstraction should Datum use
   for the first product-real slice?
2. Should terminal session metadata and scrollback persist across Datum
   application restarts, or only within a running process?
3. Should the first slice expose MCP via a local socket, CLI discovery only,
   or both?
4. Should terminal-launched command history be recorded in local session
   history, project history, or only proposal/artifact/check provenance when it
   affects design state?
5. How should Datum handle missing `$SHELL`, unsupported shells, or non-Linux
   platforms in early builds?
6. What minimum copy/paste and selection behavior is required before the
   terminal can ship as product-real?
