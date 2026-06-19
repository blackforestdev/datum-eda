# Spec Parity Contract

This file is the machine-checkable bridge between implemented code surfaces and
human specifications. It does not replace the domain specs. It records compact
inventories that must be updated when code changes an implemented surface.

The inventory source definitions live in `specs/spec_parity_manifest.json`.
The gate is `python3 scripts/check_spec_parity.py`, wired through
`scripts/run_drift_gates.sh`.

## Policy

- Specs remain authoritative for product intent and acceptance criteria.
- Code-derived inventories are authoritative for implemented surface shape.
- A change that adds, removes, or renames an implemented surface must update the
  relevant spec and refresh this inventory in the same change.
- The digest is over the sorted inventory item names, one per line.
- These counts are freeze points, not completion claims.

## Inventories

| Inventory | Owner Spec | Count | SHA256 |
|-----------|------------|-------|--------|
| `mcp_runtime_methods` | `specs/MCP_API_SPEC.md` | 75 | `0658b5fe73c2650a12f8d46d888612e4e495fa2c64516dd6e018767076bf5c7d` |
| `cli_project_commands` | `specs/PROGRAM_SPEC.md` | 182 | `5825b5ada7e9f028c35eba7480efac8bd1fa748358537cf7b79929ed6a907411` |
| `engine_text_modules` | `docs/gui/DATUM_TEXT_ENGINE_PHASE_2_IMPLEMENTATION_PLAN.md` | 11 | `1233903bce862aa7ef22879e67e8cbef3bae2bf5e823bff9e53f39b4735c8059` |
| `m7_text_visual_fixtures` | `docs/gui/DATUM_TEXT_ENGINE_FIDELITY_FIXTURES.md` | 4 | `7099fa49aca6e9574dc7ea5847914d8c20969222b5f2cf79090016539044e107` |
| `workspace_crates` | `specs/PROGRESS.md` | 7 | `2ba0685f5e07398f9fa04025c000cf4b453ae21f17e5c769524fe8e23d2a5d69` |
| `daemon_dispatch_methods` | `specs/PROGRESS.md` | 53 | `8012a535e97f1c634c2ff24d303d9659aa9ee4c2ef7977301d51a462e12c778f` |
| `engine_api_pub_fns` | `specs/ENGINE_SPEC.md` | 64 | `98e2f42b17e47de83118e2b8f15779d80dd0087d265a74ee49310d15c9093b06` |
