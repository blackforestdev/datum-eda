# Historical structural stabilization

Status: historical. This summary consolidates the former decomposition plan,
decomposition backlog and stabilization log. It supplies no current task,
execution grant, source-size allowance, test requirement or milestone acceptance.

The March 2026 structural track sought behavior-preserving extraction of runtime
and test monoliths into domain-owned modules. The March 27 discovery identified
CLI command_project (9,244 lines), main (3,030 pre-test lines), command_exec
(2,595), cli_args (2,262), engine export (928 pre-test) and board (716 pre-test)
as high-risk concentrations. Those are dated observations, not current sizes.

The stabilization log reported initial guardrail closure, high-risk inline-test
extraction, production decomposition and prevention work completed. Its later
repo-wide hotspot phase still said In Progress; this consolidation does not
retroactively close it. Recorded work included API/import/board/connectivity/
ERC/DRC/pool extraction, smaller CLI and daemon test modules, test-harness
parity/performance shards and MCP support/dispatch/client test splits. An
interrupted M3 CLI extraction was repaired within that structural pass; it was
not a product scope change. Its reported passing checks were historical proof
against then-existing tooling, not current verification.

The separate decomposition plan/backlog proposed D01-D11 for CLI entry/dispatch,
project query and mutation domains, CLI arguments, export and board helpers.
They also prescribed dedicated PR lanes, obsolete numeric freeze budgets and
automatic fallback work after blocking. These prescriptions are retired. The
size-budget gate system behind them was retired on 2026-07-02. Current source
health belongs to decision 022 and [SOURCE_HEALTH_POLICY.md](SOURCE_HEALTH_POLICY.md);
current task/grant selection belongs to the structured Frontier and selector.

Full original records, including individual extraction counts, proposed write
sets, thresholds, dates, test commands and results, remain byte-for-byte in
commit `97749064`. Retrieve with `git show 97749064:<original-path>`:

- `docs/STABILIZATION_PLAN.md` (this summary replaces the detailed log).
- `docs/DECOMPOSITION_PLAN.md` (removed from the current tree).
- `docs/DECOMPOSITION_BACKLOG.md` (removed from the current tree).

[The review index](reviews/specification-history/maintenance-records.md#organization-and-evidence-cleanup) records
verified hashes and the other historical-plan dispositions. Old directory trees
in explicitly historical architecture records describe their original snapshot.
