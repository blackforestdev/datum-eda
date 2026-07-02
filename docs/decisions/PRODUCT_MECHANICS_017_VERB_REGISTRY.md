# Product Mechanics Decision 017: Single-Source Verb Registry

> **Status**: Ratified.
> **Date**: 2026-07-02.
> **Scope**: Declaration and projection of the user-facing verb surface
> (MCP tool names == GUI terminal command ids) across CLI, daemon, MCP,
> and GUI terminal catalogs.
> **Depends on**:
> `PRODUCT_MECHANICS_004` (no private mutation paths — write-surface
> classification travels with each verb),
> `docs/contracts/AI_CLI_MCP_TOOL_SURFACE.md` (one operation vocabulary
> across surfaces; a redundant tool is a defect).

## Problem

The user-facing verb surface (~530 registered tools) is mirrored by hand in
five places: the CLI clap definitions (`crates/cli/src/cli_args_*.rs`), the
daemon dispatch table (`crates/engine-daemon/src/dispatch.rs`), the MCP Python
catalogs (`mcp-server/tools_catalog_*.py`), the GUI terminal command catalog
(`crates/gui-protocol/src/terminal_command_catalog.rs`), and the Python bridge
argv builders (`mcp-server/server_runtime*.py`). Drift between the mirrors is
held back only by count/hash gates (`check_mcp_public_taxonomy`,
`mcp_runtime_methods` spec-parity digests) that detect divergence without
naming the authority. Every new verb costs five hand edits; every rename risks
a silent split between what agents can call, what the terminal advertises, and
what the CLI parses.

## Decision

1. **One declarative table.** A leaf workspace crate, `crates/verb-registry`
   (deps: `serde`/`serde_json` only), declares each verb exactly once as a
   `VerbSpec`: id (MCP tool name == GUI command id), summary, status
   (public/hidden/retired) with replacements and retirement metadata, dispatch
   (`Cli { method, argv-template }` or `DaemonRpc { method }`), ordered
   `ParamSpec`s (parameter order IS the positional dispatch-args order),
   optional raw-schema override, write-surface classification, and
   terminal-catalog visibility. Assembly is per-family modules
   (`verbs_artifact.rs`, ...), sorted by id, invariants unit-tested.
2. **Checked-in generated projection.** `datum-verb-catalog --write` emits the
   deterministic `mcp-server/datum_tool_catalog.json` (sorted verbs, sorted
   keys, trailing newline); `--check` is a drift gate registered in
   `scripts/run_drift_gates.sh`. The MCP Python side loads the JSON at import
   (`tools_catalog_generated.py`) and produces tool specs in the exact
   hand-written dict shape (`x_dispatch_method`/`x_dispatch_args`/write-surface
   keys), so runtime dispatch through the existing `server_runtime` bridge
   methods is byte-identical. Execution paths do not move in this decision —
   the registry supplies the catalog entries; the bridge remains the executor.
3. **Behavioral tests, not prose.** A clap round-trip integration test
   (`crates/cli/tests/verb_registry_roundtrip.rs`) renders every
   `Dispatch::Cli` argv template with dummy values and asserts the real
   `datum-eda` clap surface accepts it, so argv templates cannot drift from
   the CLI.

## Migration

Families migrate one prefix at a time through the `MIGRATED_PREFIXES`
frozenset in `tools_catalog_generated.py`: a migrated prefix is owned by the
generated catalog, its hand-written entries are deleted, and a duplicate tool
name between generated and hand-written sources raises at import (caught by
`server.py --self-test`). Seeded now: `datum.artifact` (11 verbs). Target: all
registered verbs, at which point the hand-written catalog modules and the
count-based taxonomy constants collapse into registry projections, and the
GUI terminal catalog and daemon dispatch table become generated consumers.
Existing gates (`check_mcp_public_taxonomy`, spec parity) keep their
invariants unchanged throughout — they verify the merged surface, whatever
its source.
