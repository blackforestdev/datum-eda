# Datum Terminal Agent Interoperability Specification

Status: governed product contract

Controlling decisions: Product Mechanics 027 and 028.

## 1. Scope and success condition

This contract closes the gap between **can run in the terminal** and **can use
Datum deliberately**. Terminal emulation owns PTY correctness. The
interoperability plane owns client launch, MCP registration, context, authority,
portable workflows, and proof. Neither may impersonate the other.

An adapter is supported only when a fresh installation can be launched from a
Datum project and complete the governed inspect→propose→review→apply→refresh
round trip without hidden global configuration, a remembered prompt, a private
write path, or terminal-screen scraping.

## 2. Architecture

```text
agent TUI <-> Datum PTY
                  |
        AgentAdapterRegistry
          |               |
  discovery/config   MCP client registration
          |               |
      datum-eda CLI  datum-eda MCP stdio broker
          \               /
             Datum daemon
                  |
       typed operations + journal
```

The agent remains replaceable. Datum-specific intelligence lives in the
canonical CLI/MCP/context contracts, not in vendor launch strings.

## 3. Discovery document

The existing `datum_terminal_context_v1` remains the context envelope. A new
`datum_agent_discovery_v1` runtime document points to it and describes how a
client connects. It is written atomically under a protected `.datum/runtime/`
session directory and is never tracked.

Required fields:

- schema version, project root/id, terminal session id, agent launch id;
- live context path/id and explicitly pinned context path/id;
- current and pinned model revisions plus accepted transaction tip;
- canonical `datum-eda` executable and context refresh command;
- MCP profiles with transport, command/argv or loopback URL, protocol range,
  declared capabilities, expiry, and credential-descriptor reference;
- granted Datum capabilities and approval policy;
- applicable project-instruction roots and portable workflow inventory;
- adapter id/version, agent executable/version, resume identity, and lifecycle
  event path.

The document contains no bearer token, API key, OAuth refresh token, shell
history, or copied agent credential. Unknown required schema versions fail
closed with an actionable `agent doctor` result.

## 4. Agent adapter registry and launcher

<!-- REQ:TERMINAL-T4A-AGENT-LAUNCH:AI-DISC-01 -->
<!-- EVIDENCE:TERMINAL-T4A-AGENT-LAUNCH:AI-DISC-01-CLOSED -->
<!-- REQ:TERMINAL-T4A-AGENT-LAUNCH:AI-DISC-02 -->
<!-- EVIDENCE:TERMINAL-T4A-AGENT-LAUNCH:AI-DISC-02-CLOSED -->
`datum-eda agent list`, `datum-eda agent doctor <adapter>`, and
`datum-eda agent launch <adapter> [-- <native args>...]` are the canonical
surface. Direct shell launch remains supported but is reported as unverified
until the client independently registers Datum MCP.

Each adapter record declares:

- stable adapter id and supported version range;
- binary lookup/version probe and native interactive/resume arguments;
- project cwd and environment allowlist;
- native MCP configuration shape and approval requirement;
- project instruction/rule files and optional skill/plugin roots;
- ephemeral config injection strategy and cleanup behavior;
- capability limitations, known deltas, and verification fixture.

Required profiles are `codex`, `claude-code`, `cursor-cli`, and
`local-generic`. `local-generic` provides discovery/CLI and a printed standard
MCP command without pretending an unknown client supports automatic setup.

<!-- REQ:TERMINAL-T4A-AGENT-LAUNCH:AI-DISC-03 -->
The launcher uses a protected per-session directory and shows the executable,
cwd, MCP server name, capability grant, and config lifetime before launch. It
must not rewrite user-global configuration. Persistent project setup is a
separate explicit install/remove operation with a reviewable diff.

## 5. MCP connection and capability contract

### 5.1 Primary stdio broker

The portable connection is a standard MCP stdio subprocess equivalent to:

```bash
datum-eda mcp serve --discovery "$DATUM_AGENT_DISCOVERY"
```

Stdout carries only MCP protocol messages; stderr carries bounded structured
logs. The broker validates project/session scope before connecting to the
internal Datum daemon. Client exit tears down its broker and revokes its lease.

### 5.2 Optional Streamable HTTP

Shared long-lived access may use Streamable HTTP only on loopback with Origin
validation, negotiated protocol version, scoped authentication, expiry,
revocation, bounded sessions, and no token passthrough. It is never required for
local agent compatibility.

### 5.3 Server primitives

Tools retain the `datum.<group>.<verb>` taxonomy. Resources include at minimum:

- `datum://project/current`
- `datum://context/live`
- `datum://context/pinned/{context_id}`
- `datum://model/revision/{revision}`
- `datum://selection/{context_id}`
- `datum://checks/current` and `datum://check/{fingerprint}`
- `datum://proposal/{proposal_id}`
- `datum://artifact/{artifact_id}`
- `datum://render/board/{revision}.svg`
- `datum://render/schematic/{revision}.svg`
- resource templates for stable object IDs and paginated collections.

When negotiated, the server supports resource list-change and update
notifications. Clients without subscriptions use explicit refresh. Prompts are
user-invoked workflow templates and never bypass tool approval.

## 6. Context and concurrency

`live_context` follows GUI state. `pinned_context` is immutable for an agent work
unit. Launch pins the initial context; an explicit agent/user action may pin a
new one. Changing pane focus or selection never retargets an existing request.

Every proposal/apply envelope includes:

- agent launch/session id and actor identity;
- pinned `context_id`;
- `expected_model_revision` and accepted transaction tip;
- stable selected/object IDs, never positional selection alone;
- requested capability and approval provenance.

A mismatch returns a structured stale-context result with current revision,
affected IDs, and refresh/rebase options. Silent last-write-wins is prohibited.

## 7. Authority and security

Capability profiles are cumulative and explicit:

1. `inspect`: resources, queries, checks, and non-mutating artifacts;
2. `propose`: create/preview/validate proposals;
3. `apply-approved`: apply only owner-reviewed proposals;
4. `unattended`: narrowly scoped owner policy with limits and revocation.

The default is inspect+propose. Filesystem/network/shell permissions remain
owned by the agent harness and operating system; Datum grants only Datum-domain
authority. Every committed design mutation records agent, launch/session,
context, expected revision, approval, operation batch, diff, and journal result.

## 8. Portable workflows and client projections

The canonical workflow inventory binds one intent to CLI, MCP tool/resource/
prompt, required capabilities, context inputs, review gate, and evidence. Thin
client projections may improve discovery:

- Codex: repository instructions plus optional skills/plugins;
- Claude Code: controlling instructions plus project MCP configuration;
- Cursor-compatible clients: project rules plus MCP configuration;
- local agents: standard MCP command and discovery schema.

Projection files never redefine workflow semantics. A parity gate detects a
missing, stale, or extra privileged projection. Failure to load a proprietary
skill cannot remove the underlying CLI/MCP workflow.

## 9. Shell integration boundary

OSC 7 may report cwd and OSC 133 may mark prompt/command/output boundaries for
session UX, history, and new-session-in-cwd behavior. These signals are
untrusted presentation metadata. Datum does not parse terminal output into a
typed design operation, auto-run a discovered command, or infer approval.

## 10. Verification matrix

Each required adapter must prove on a production Datum build:

1. version probe, launch and native TUI behavior;
2. correct project cwd and discovery identity;
3. native MCP registration and enumeration of tools/resources/prompts;
4. pinned-context read of the selected stable object;
5. proposal creation and deterministic preview with no mutation;
6. owner review followed by an authorized apply and journal evidence;
7. revision-change observation, stale-request refusal, and refresh;
8. resource update or explicit-refresh behavior;
9. terminal/agent restart and native resume without context confusion;
<!-- REQ:TERMINAL-T4A-AGENT-LAUNCH:AI-DISC-04 -->
10. missing client, rejected project config, expired credential, daemon loss,
    unsupported feature, teardown, and revocation behavior.

The matrix records exact client/core versions and artifacts. Launch success
alone is a failing result.

## 11. Delivery ownership

- T4a owns §3–4.
- T4b owns §5.
- T4c owns §6–7.
- T4d owns §8–10 and the OSC boundary in §9.
- Existing T4 verification consumes all four; it does not absorb or waive them.

No T4a–T4d issue closes on documentation-only or mocked-terminal evidence.
