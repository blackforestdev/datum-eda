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
| `mcp_runtime_methods` | `specs/MCP_API_SPEC.md` | 186 | `b7d70db390667030975a34e1a4ff4413bcda26a333ba62d2b0a39f0255098590` |
| `cli_project_commands` | `specs/PROGRAM_SPEC.md` | 271 | `51c201fe22fafc063ff2a7a46c9b1303e94088608743da9d637d86cf3da54ffe` |
| `engine_text_modules` | `docs/gui/DATUM_TEXT_ENGINE_PHASE_2_IMPLEMENTATION_PLAN.md` | 11 | `1233903bce862aa7ef22879e67e8cbef3bae2bf5e823bff9e53f39b4735c8059` |
| `m7_text_visual_fixtures` | `docs/gui/DATUM_TEXT_ENGINE_FIDELITY_FIXTURES.md` | 4 | `7099fa49aca6e9574dc7ea5847914d8c20969222b5f2cf79090016539044e107` |
| `workspace_crates` | `specs/PROGRESS.md` | 7 | `2ba0685f5e07398f9fa04025c000cf4b453ae21f17e5c769524fe8e23d2a5d69` |
| `daemon_dispatch_methods` | `specs/PROGRESS.md` | 54 | `ba9f30a2b8465dec68897f5a88f31a6c8d2f8406589c99bb5df580cfc8bc0717` |
| `engine_api_pub_fns` | `specs/ENGINE_SPEC.md` | 65 | `50cb7c23460e05ff971a9c264ab97744203de31e79e2b75473d0bd78b9557a3b` |
