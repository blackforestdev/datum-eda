# Datum GUI Write-Path Enablement Plan

> **Status**: Governed active planning plan.
> **Authority**: Realizes the "GUI Edit Authority" mandate of
> `docs/decisions/PRODUCT_MECHANICS_019_GUI_PRODUCT_MODEL.md` and the
> single-source dispatch model of
> `docs/decisions/PRODUCT_MECHANICS_017_VERB_REGISTRY.md`.
> **Companion to**: `docs/gui/DATUM_GUI_MENU_BINDINGS.md` (the gap this plan
> closes) and `docs/gui/DATUM_GUI_PRODUCT_SPEC.md`.
> **Scope**: The four-item plumbing that lets the GUI author journaled engine
> operations directly, so authoring menu items and editor tools can be wired
> without the terminal CLI-string hack decision 019 forbids.

## Context

The 2026-07-05 capability inventory (`DATUM_GUI_MENU_BINDINGS.md`) established
that the engine already has the capability behind nearly every menu item (338
verbs, native-write builders, full CLI), but the GUI has **no clean path to reach
a journaled operation**. Reads work via `run_cli_json`; every authoring action
falls back to the terminal CLI-string handoff. Four converging causes block the
correct path. This plan bounds the work to remove them.

This is a **planning artifact**: it specifies what execution must demonstrate.
Each workstream below is feature execution and requires explicit authorization
before implementation begins.

## Invariant this enables

A GUI gesture or property edit resolves to: GUI action model → verb + typed
params → daemon `native.write` (journaled `commit()`) **or** a proposal → journal
append → resolver refresh. No CLI string is synthesized. This is decision 019's
required mutation path, made reachable.

## Workstreams

### W1 — Register the remaining native-write families with `native.write`
- **Now**: `crates/engine/src/api/native_write/registry.rs` aggregates only
  `project::VERBS` + `waivers::VERBS` = 7 verbs. The board / schematic / library /
  routing / manufacturing builders exist but have no `VERBS` table, so
  `native.write` cannot reach them.
- **Change**: add a `VERBS: &[NativeWriteVerb]` table per family (each adapter
  parses JSON params → calls the existing builder), and extend the registry
  aggregation. No new builders — this is wiring existing ones to the verb surface.
- **Touches**: `crates/engine/src/api/native_write/*` (`registry.rs` + per-family
  modules).
- **Contract update on execution**: `specs/ENGINE_SPEC.md` (engine verb-surface
  coverage), and the verb-registry parity if verb ids change.
- **Acceptance**: `native.write` can dispatch a representative verb from every
  authoring family (schematic place, board place/route, library create, output
  job) in dry-run and applied modes; per-family round-trip tests.

### W2 — Add a `NativeWrite` dispatch variant to the verb registry
- **Now**: `crates/verb-registry/src/lib.rs` `Dispatch` has only
  `Cli { method, argv }` and `DaemonRpc { method }`. Every write verb resolves to
  a CLI argv template; no verb targets `native.write`.
- **Change**: add `Dispatch::NativeWrite { verb }` (journaled-commit path) and
  tag each journaled write verb with it, so a caller (GUI) learns from the
  registry that a verb is authored through `native.write`, not a CLI string.
- **Touches**: `crates/verb-registry/` + the generated catalog projection.
- **Contract update on execution**: `docs/decisions/PRODUCT_MECHANICS_017_VERB_REGISTRY.md`
  (Dispatch model gains a third variant) — same-change edit.
- **Acceptance**: catalog round-trip green with the new variant; drift gate
  (`datum-verb-catalog --check`) passes; write verbs report `NativeWrite`.

### W3 — Expose verb / param-schema enumeration on the daemon
- **Now**: `native.describe` returns only `{ project_root, project_id,
  project_name, model_revision, journal_len }` — no verb list, no schemas. The
  engine has `native_write_verbs()` but it is not exposed.
- **Change**: extend `native.describe` (or add `native.list_verbs` /
  `native.describe_verb`) to return the verb ids + param schemas from
  `native_write_verbs()`, so the GUI can discover available operations and build
  parameter forms dynamically.
- **Touches**: `crates/engine-daemon/src/dispatch.rs` + `main.rs`.
- **Contract update on execution**: `specs/MCP_API_SPEC.md` (daemon method
  catalog) — same-change edit.
- **Acceptance**: a client can enumerate the authoring verb set and one verb's
  param schema over the socket; covered by a daemon test.

### W4 — Give `gui-app` a daemon client + GUI action model
- **Now**: `gui-app` does not link the engine (dev-dep only) and has no
  Unix-socket/JSON-RPC client; authoring goes through the terminal string handoff.
- **Change**: add a JSON-RPC-over-Unix-socket client to `gui-app`; a GUI action
  model that, for a user gesture, assembles `{ verb, params, reason, source }`
  and calls `native.write` (or creates a proposal), then refreshes scene state
  through the resolver. Retire the terminal-lane authoring handoff for menu/tool
  actions.
- **Touches**: `crates/gui-app/`, `crates/gui-protocol/` (action model types).
- **Contract update on execution**: realizes `DATUM_GUI_PRODUCT_SPEC.md`
  "Editor Authority"; update its status.
- **Acceptance**: one menu action commits end-to-end through the daemon with a
  journal entry and no CLI string, proven by a behavioral test + screenshot.

## Sequencing (front-load the risk-reducing proof)

1. **P0 — thin vertical proof.** Build the W4 client core and call `native.write`
   for a verb already wired today (`datum.project.set_name`, one of the 7). Prove
   a single journaled GUI edit (rename project) end-to-end: gesture → daemon →
   `commit()` → journal → GUI refresh, **no CLI string.** This de-risks the whole
   pipe before widening it.
2. **W1** — register the remaining families, widening `native.write` coverage
   from 7 verbs to the full authoring set.
3. **W2** — add the `NativeWrite` dispatch variant so the GUI builds actions from
   the registry generically instead of per-verb hardcoding.
4. **W3** — add daemon enumeration so the GUI discovers verbs/schemas dynamically
   (enables generic property forms and future tools without GUI code per verb).

P0 proves the path; W1 widens it; W2 makes it registry-driven; W3 makes it
self-describing. Each step is independently shippable and testable.

## Non-goals

- Not building the full menu bar or any editor UI — this is the write *path*, not
  the surfaces on top of it.
- Not changing the operation vocabulary, `commit()`, journal, or proposal model
  (decision 001/004) — only how the GUI reaches them.
- Not touching the frozen imported-KiCad converter session (decision 011).

## Governance

- **Status home**: tracked in `specs/PROGRESS.md` → "Scope Integration /
  Substrate Readiness" (the active-work surface), one row, current-vs-target.
- **Classification**: governed in `specs/spec_governance_manifest.json`.
- **Cross-spec parity on execution**: W2 edits decision 017; W3 edits
  `specs/MCP_API_SPEC.md`; W1 edits `specs/ENGINE_SPEC.md`; W4 updates
  `DATUM_GUI_PRODUCT_SPEC.md` — each in the same change as its implementation.
- **Not owned by `specs/PROGRAM_SPEC.md`**: that file defines legacy milestone
  contracts (M0–M4) and delegates all status to PROGRESS; its "v1 does not
  provide a GUI" text predates the course correction and needs its own owner
  reconciliation (reconcile to the product-mechanics model or classify its
  milestone-roadmap function historical) — this plan does not fold GUI work into
  legacy milestone framing.
