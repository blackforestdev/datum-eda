# Product Mechanics 028: Terminal Agent Discovery And Interoperability

Status: ratified doctrine

## Decision

Datum's fully fledged terminal is the universal compatibility lane for code
agents, but a correct PTY does not by itself integrate an agent with Datum. An
agent is product-integrated only when its native client discovers the active
project, a standard MCP connection, current and pinned context, capability
scope, portable workflows, and revision-safe Datum operations without relying
on a launch prompt or prior conversational knowledge.

Datum therefore owns an **agent interoperability plane** beside—not inside—the
terminal emulator. Agents remain ordinary user-controlled PTY processes. A
Datum launcher prepares ephemeral client-native configuration and discovery;
a standard MCP broker exposes tools, resources, and prompts; the canonical CLI
remains an equivalent manual/script surface; and every mutation still uses the
one typed operation, proposal, commit, and journal authority.

This decision extends Product Mechanics 027 FT-009. It does not create a built-
in model, private agent protocol, privileged edit path, or terminal-output
screen scraper.

## Normative Rules

- **AI-001 — launch is explicit and ordinary.** `datum-eda agent launch`
  starts the selected installed agent as an ordinary child of the user's shell
  in the real Datum PTY. Directly typing `codex`, `claude`, or another command
  remains valid. Datum never requires a special agent tab.
- **AI-002 — closed adapter registry.** Each supported adapter declares its
  executable/version probe, argument and resume behavior, project-instruction
  convention, MCP configuration mechanism, environment allowlist, and teardown
  behavior. Codex, Claude Code, a Cursor-compatible CLI, and a generic local
  agent are required profiles. Vendor behavior is isolated behind the registry.
- **AI-003 — ephemeral configuration.** Launch adapters write only protected,
  per-session runtime configuration unless the owner explicitly installs a
  persistent project configuration. They do not silently edit user-global agent
  settings, credentials, repository instructions, or skills.
- **AI-004 — standard MCP boundary.** The primary local agent connection is a
  standard MCP stdio broker whose stdout contains only protocol messages and
  whose logs use stderr. Datum's internal daemon socket remains an implementation
  detail. Optional Streamable HTTP binds only to loopback and requires origin
  validation plus scoped authentication. `DATUM_MCP_ENDPOINT` alone is discovery
  metadata, not proof that a client registered the server.
- **AI-005 — full MCP primitives.** Datum exposes typed tools for queries and
  actions, stable `datum://` resources for project/model/selection/check/
  proposal/artifact/render context, resource templates and negotiated change
  notifications, and user-invoked prompts for portable workflows. Capability
  negotiation is honest; unsupported client features retain CLI equivalents.
- **AI-006 — live versus pinned context.** Live GUI focus and selection may
  change while an agent reasons. Every work unit therefore pins an immutable
  `context_id` plus `model_revision`; live context remains separately refreshable.
  No operation silently retargets itself to a later GUI selection.
- **AI-007 — optimistic mutation fence.** Proposal and apply requests carry the
  pinned context identity and expected model revision. Stale requests refuse
  with a structured rebase/refresh path unless an explicitly reviewed operation
  contract proves safe rebasing.
- **AI-008 — scoped authority.** Agent sessions receive explicit `inspect`,
  `propose`, `apply-approved`, or owner-enabled unattended capabilities.
  Inspect/propose is the default. Credentials are short-lived, project/session-
  bound, revocable, absent from tracked discovery files and telemetry, and never
  grant raw shard or journal writes.
- **AI-009 — portable workflow authority.** Datum workflows are canonical in
  CLI commands plus MCP tools/resources/prompts. Codex skills, Claude facilities,
  Cursor rules, and local-agent packages are optional thin projections, never
  the only implementation of a workflow. Their inventory is generated or parity-
  checked so vendor projections cannot drift semantically.
- **AI-010 — instruction discovery.** The launcher tells each client where its
  native project instructions live and verifies that they were eligible for
  loading. It does not assume one client's instruction or skill format works in
  another client.
- **AI-011 — shell metadata is non-authoritative.** OSC 7 working-directory and
  OSC 133 prompt/command boundaries may improve session UX. Datum never converts
  terminal text, escape output, or inferred commands into design operations.
- **AI-012 — product proof.** Support is claimed per adapter only after a
  production build passes launch, discovery, MCP enumeration, pinned-context
  inspect, proposal, owner review, apply, journal/revision observation, context
  refresh, resume, failure, and teardown tests. Merely opening the agent is not
  acceptance.

## Implementation Slices

1. **T4a launcher/discovery:** adapter registry plus `agent list`, `doctor`, and
   `launch`; protected per-session config and lifecycle.
2. **T4b MCP interoperability:** stdio broker, optional secured loopback HTTP,
   tools, resources, templates, subscriptions, and prompts.
3. **T4c context/authority:** live/pinned context, revision fences, scoped
   capabilities, credentials, audit, and structured stale-state handling.
4. **T4d workflow parity:** canonical workflow inventory, checked client
   projections, OSC metadata boundary, and the named agent round-trip matrix.
5. **T4 final verification:** closes the full terminal epic only after these
   slices and the terminal capability phases are complete.

T0 shell-truth work remains valid. This decision is a specification barrier
after T0-C01; once the barrier is closed, T0 resumes at C02. T4 cannot close or
be bypassed without T4a–T4d.

## Interoperability-contract completion anchors

<!-- REQ:TERMINAL-AGENT-INTEROP-CONTRACT:AIC-C01 -->
- **AIC-C01 — audit discovery truth.** Reconcile the existing context envelope,
  launch prompts, MCP/CLI surface, agent configuration mechanisms, and missing
  proof boundaries.
<!-- EVIDENCE:TERMINAL-AGENT-INTEROP:AIC-C01-AUDIT -->

<!-- REQ:TERMINAL-AGENT-INTEROP-CONTRACT:AIC-C02 -->
- **AIC-C02 — ratify the product contract.** Land this decision and the governed
  `DATUM_TERMINAL_AGENT_INTEROP_SPEC.md`.
<!-- EVIDENCE:TERMINAL-AGENT-INTEROP:AIC-C02-CONTRACT -->

<!-- REQ:TERMINAL-AGENT-INTEROP-CONTRACT:AIC-C03 -->
- **AIC-C03 — seed bounded implementation.** Create T4a–T4d issues with exact
  acceptance criteria and terminal-epic ownership.
<!-- EVIDENCE:TERMINAL-AGENT-INTEROP:AIC-C03-TRACKER -->

<!-- REQ:TERMINAL-AGENT-INTEROP-CONTRACT:AIC-C04 -->
- **AIC-C04 — enforce sequence.** Reconcile Frontier and hard tracker edges so
  T4 depends on the complete agent-interoperability chain.
<!-- EVIDENCE:TERMINAL-AGENT-INTEROP:AIC-C04-GRAPH -->

<!-- REQ:TERMINAL-AGENT-INTEROP-CONTRACT:AIC-C05 -->
- **AIC-C05 — validate and hand back.** Pass governance and source-health gates,
  close with commit evidence, and explicitly return the selector to T0-C02.
<!-- EVIDENCE:TERMINAL-AGENT-INTEROP:AIC-C05-HANDOFF -->
