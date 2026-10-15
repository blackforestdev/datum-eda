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
| Native design delivery path | [Product-outcome view](../specs/progress/production_foundation_details.md#native-product-outcomes--initial-reorganization) | Source-qualified delivered behavior, the original twenty-task snapshot with subsequent doorway completion, and retained development/qualification owners; does not select work. |
| Current delivered behavior and open qualification | [Progress](../specs/PROGRESS.md) and the selected task's evidence | Distinguish requirements, target behavior, development availability and accepted qualification. |
| Scheduled task, completion steps and grant | [Structured Frontier](../specs/active_frontier.json), queried through [project_status.py](../scripts/project_status.py) | The selector joins the roadmap to tracker and ownership state. Links do not select or authorize work. |
| Approval, review and historical proof | [Review index](reviews/README.md) | Compact findings, exact receipts, source pins and recovery references. Raw captures belong in external workspaces. |
| Research and its governed consumers | [Research traceability](RESEARCH_TRACEABILITY.md), [evidence routes](../specs/evidence_traceability_manifest.json) | Research informs requirements; a study or screenshot alone is not ratification. |
| Standing delivery rules and exact task grants | [PM059](decisions/PRODUCT_MECHANICS_059_DELEGATED_FEATURE_DELIVERY.md), [governed delivery history](reviews/specification-history/feature-delivery-records.md) | Standing delegation stays in the decision; task-specific exceptions, approvals and requirement markers retain their original scope in history. |
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

## Development tools

Run repository commands from the checkout root. This table navigates existing
entry points; contributor procedures and the selected task still own when a
mutation or verification is appropriate.

| Need | Existing entry point | Work performed |
| --- | --- | --- |
| Current task and ordered completion steps | `python3 scripts/project_status.py next` / `details` | Read-only joined roadmap/tracker/claim queries. |
| Structural checks for a focused change | `bash scripts/run_drift_gates.sh --focused` | Resource, source, authority, parity, traceability and boundary checks; no Cargo compilation. |
| Deliberate full qualification | `bash scripts/run_drift_gates.sh --full` | Compiles and runs workspace, proof and GUI checks; produces output. It is also the default mode. |
| Guarded Rust work | [run_cargo_guarded.py](../scripts/run_cargo_guarded.py) | Serializes Cargo and checks disk/target policy; select `--workload interactive` or `proof` as required by contributor rules. |
| Staged formatting check | `python3 scripts/check_rustfmt.py --staged` | Nonmutating check of staged Rust files. |
| Format selected Rust files | [format_rust.py](../scripts/format_rust.py) | Writes only explicitly named paths; inspect shared-tree status before and after. |
| Documentation and ownership checks | [check_spec_governance.py](../scripts/check_spec_governance.py), [check_spec_parity.py](../scripts/check_spec_parity.py), [check_evidence_traceability.py](../scripts/check_evidence_traceability.py) | Checks classification, inventory parity and evidence freshness; printing a digest does not accept it. |
| Inspect cache use | `python3 scripts/manage_cargo_cache.py` | Reports cache inventory; deletion requires explicit selected paths and `--apply`. |
| Preview released build-output cleanup | `python3 scripts/prune_development_output.py --build-only` | Preview only; native inputs/runtime retention and apply/release rules are described in the [review index](reviews/README.md). |
| Locate scratch or recovered fixture paths | [development_workspace.py](../scripts/development_workspace.py) | `scratch <family>` and `resolve <old-path>` print external paths; they do not create fixtures or authorize replay. |
| Tracker operations | [Operational tracker procedures](AGENT_OPERATIONAL_PROCEDURES.md#issue-tracker-beads-br) | Use `br`; claims, exports and closures mutate tracker state and must stay synchronized with the Frontier. |
| Manual diagnostics and older proof recipes | [scripts](../scripts/) and [review records](reviews/) | Inspect each tool's source/help and recorded prerequisites. A filename prefix or preserved recipe is not an instruction to execute it. |

`check_*.py` files are individual gate entry points; `test_*.py` files exercise
tooling and boundaries; `run_*` files orchestrate work; generators/formatters
may write files. These names aid discovery, not side-effect guarantees.
`check_alignment.py` does nothing without an explicit mode; `--run-health`
prebuilds matching companions and runs workspace tests, while `--run-gates`
runs executable milestone gates. Use the focused runner for the documented
structural-only batch instead of discovering this distinction by trial.

Historical WDQ tooling remains manual diagnostics/history under PM044. Normal
contribution uses the existing Git hooks and current gates; preserved enrollment
or promotion scripts do not reinstate retired blocking enforcement. No tools
are moved or renamed by this navigation, so imports and existing callers remain
stable.

## Portable references

Use relative links for files in this checkout. Historical source-line citations
use a commit-pinned repository permalink; the cited offsets belong to that
revision, not the refactored current file. Do not infer current implementation
or qualification from a historical code citation. The repaired older citations
use the newest retained document revision where the referenced source offsets
exist, preserving a retrievable source basis without inventing new line numbers.

The [research index](../research/README.md) distinguishes topic/integration
history from the complete evidence-route inventory and current scheduled work.
Its existing source-analysis symlinks are research references; this pass neither
creates sandboxes nor imports their code.
