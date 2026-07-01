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
| `mcp_runtime_methods` | `specs/MCP_API_SPEC.md` | 182 | `8b6254b4bb9ab3ac6da3b0f9d9566ae9bbfde4deb6618e494ebadf2361d2e211` |
| `cli_project_commands` | `specs/PROGRAM_SPEC.md` | 286 | `86eb5b27881684a8340972b377e47e831d56806ccccb018485c8cd36a0162730` |
| `engine_text_modules` | `docs/gui/DATUM_TEXT_ENGINE_PHASE_2_IMPLEMENTATION_PLAN.md` | 11 | `1233903bce862aa7ef22879e67e8cbef3bae2bf5e823bff9e53f39b4735c8059` |
| `m7_text_visual_fixtures` | `docs/gui/DATUM_TEXT_ENGINE_FIDELITY_FIXTURES.md` | 4 | `7099fa49aca6e9574dc7ea5847914d8c20969222b5f2cf79090016539044e107` |
| `workspace_crates` | `specs/PROGRESS.md` | 7 | `2ba0685f5e07398f9fa04025c000cf4b453ae21f17e5c769524fe8e23d2a5d69` |
| `daemon_dispatch_methods` | `specs/PROGRESS.md` | 35 | `b5c5a3d7e129d18d4a45a8f1a9f329e1c4741fc687070dd0bf9eb0841ae74187` |
| `engine_api_pub_fns` | `specs/ENGINE_SPEC.md` | 65 | `50cb7c23460e05ff971a9c264ab97744203de31e79e2b75473d0bd78b9557a3b` |
| `standards_check_surface` | `specs/CHECKING_ARCHITECTURE_SPEC.md` | 29 | `56e6d1bca3d5e3245655ab9e4f5089013b0b1368156a4b7303aabd394550f2af` |
| `pool_library_surface` | `docs/contracts/LIBRARY_AUTHORING_TOOL_CONTRACT.md` | 114 | `edb4bf8faf38c8007792ce1e0c47760e18a87d2fdfb0a5db45e1b9ca844c22e6` |
