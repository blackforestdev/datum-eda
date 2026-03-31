# MCP Server Layout

This directory is intentionally split to keep runtime logic and tests isolated.

- `server.py`: thin entrypoint. Runs stdio server by default and `--self-test`.
- `server_runtime.py`: runtime transport + JSON-RPC/MCP host wiring.
- `tools_catalog.py`: exported `tools/list` catalog payload from shared tool specs.
- `tool_dispatch.py`: generic `tools/call` dispatch derived from shared tool specs.
- `test_daemon_client.py`: JSON-RPC request/response and socket transport tests.
- `test_protocol_catalog.py`: `initialize`/`ping`/notifications + `tools/list` tests.
- `test_dispatch_core.py`: core `tools/call` dispatch tests.
- `test_dispatch_write_basics.py`: write-operation dispatch + immediate follow-up behavior tests.
- `test_dispatch_read_surface.py`: read/query dispatch behavior tests.
- `test_dispatch_write_mutations.py`: stateful mutation/parity behavior tests.
- `test_dispatch_queries.py`: schematic/query/check dispatch tests.
- `test_dispatch_stateful.py`: compatibility shim for split stateful tests.
- `test_support.py`: shared fake daemon fixture used across test modules.
- `test_server.py`: compatibility shim (no tests; legacy module path only).

## Development Rules

- Keep `server.py` as composition only; no runtime business logic.
- Add new MCP tool metadata once in `tools_catalog_data.py`.
- Add any required dispatch-order/default overrides in that same shared spec entry.
- Add/adjust tests in the relevant `test_*.py` module for each behavior change.

## Local Validation

```bash
python3 mcp-server/server.py --self-test
python3 scripts/check_alignment.py
```
