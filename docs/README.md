# Documentation map

Use this map to find an existing owner. It adds no product requirements,
implementation status, scheduling rules or execution permission.

| What you need | Start here | Role |
| --- | --- | --- |
| Product behavior and ratified decisions | [Product mechanics](DATUM_PRODUCT_MECHANICS.md), [decisions](decisions/) | Product authority; preserve ratified meaning and its qualification limits. |
| Designer workflows and domain behavior | [Domain tool contracts](contracts/) | Schematic, PCB, library, checks, manufacturing, shared tool surface and UI layout contracts. |
| Detailed types, persistence and interfaces | [Program specification ownership map](../specs/PROGRAM_SPEC.md#ownership-and-precedence) | Names the formal subject owners and their inherited precedence. Directory names alone do not establish authority. |
| Canonical IR identity and invariants | [Canonical IR](CANONICAL_IR.md) | Retains governed ownership until formally promoted or subsumed; its decision reconciliations remain controlling at their stated scope. |
| Engine architecture rationale | [Engine design](ENGINE_DESIGN.md) | Explains design choices and links to the owned contracts; examples do not establish current API support. |
| Current delivered behavior and open qualification | [Progress](../specs/PROGRESS.md) and the selected task's evidence | Distinguish requirements, target behavior, development availability and accepted qualification. |
| Scheduled task, completion steps and grant | [Structured Frontier](../specs/active_frontier.json), queried through [project_status.py](../scripts/project_status.py) | The selector joins the roadmap to tracker and ownership state. Links do not select or authorize work. |
| Approval, review and historical proof | [Review index](reviews/README.md) | Compact findings, exact receipts, source pins and recovery references. Raw captures belong in external workspaces. |
| Research and its governed consumers | [Research traceability](RESEARCH_TRACEABILITY.md), [evidence routes](../specs/evidence_traceability_manifest.json) | Research informs requirements; a study or screenshot alone is not ratification. |
| Contributor instructions | [CLAUDE.md](../CLAUDE.md), [AGENTS.md](../AGENTS.md), [operational procedures](AGENT_OPERATIONAL_PROCEDURES.md) | Entry rules and action-specific procedures. Load the sections relevant to the task. |
| Historical launch and structural plans | [Architecture context](PROJECT_ARCHITECTURE_REFERENCE.md), [stabilization history](STABILIZATION_PLAN.md) | Dated context and retrieval, without current sequencing or grants. |

The [governance manifest](../specs/spec_governance_manifest.json) distinguishes
`governed`, `doctrine`, `pending` and `historical`. Classification describes the
document's role; it does not prove implementation or acceptance. Some documents
contain separately labeled current, target, compatibility and historical sections.
Read those labels before treating an example as a requirement.

For one feature, follow the domain contract to its detailed specification and
selected completion contract. Then inspect its implementation owner and tests:

| Implementation area | Source and test entry |
| --- | --- |
| Engine model, mutation and persistence | [Engine](../crates/engine/src/); tests beside the owner and under its `tests` modules |
| CLI behavior and workflow tests | [Commands](../crates/cli/src/commands/), [CLI tests](../crates/cli/src/main_tests/) and [integration tests](../crates/cli/tests/) |
| GUI behavior and presentation | [Application](../crates/gui-app/src/), [protocol](../crates/gui-protocol/src/), [renderer](../crates/gui-render/src/); tests beside those owners |
| Daemon and MCP transport | [Daemon](../crates/engine-daemon/src/), [MCP adapter and tests](../mcp-server/) |
| Shared proof and fixture tooling | [Test harness](../crates/test-harness/src/), [repository checks](../scripts/) |

A test's location identifies where to investigate; its existence does not prove
that the selected feature's acceptance criteria passed. Reuse adequate evidence
and run only the checks needed by the change and its required gates.

Native user projects belong in their chosen project directories. Generated test
projects and captures use the external workspace conventions in the
[review index](reviews/README.md#cargo-output-and-native-workspace-separation).
`target/` is disposable Cargo output. `.datum/history-cleanup/` is protected local
recovery metadata; a native project's own `.datum/` journal remains part of that
project and must be preserved with it.
