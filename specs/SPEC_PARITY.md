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
| `mcp_runtime_methods` | `specs/MCP_API_SPEC.md` | 188 | `bb9f3edf3a5b5a4e9ed0efca1fac689a842b01de7a4a9816b743e545f577a5f2` |
| `cli_project_commands` | `specs/PROGRAM_SPEC.md` | 287 | `40ef328f49a6dc21e44625d65999da45e0d320816de3ea236529f9f8d6c69dc9` |
| `engine_text_modules` | `docs/gui/DATUM_TEXT_ENGINE_PHASE_2_IMPLEMENTATION_PLAN.md` | 11 | `1233903bce862aa7ef22879e67e8cbef3bae2bf5e823bff9e53f39b4735c8059` |
| `m7_text_visual_fixtures` | `docs/gui/DATUM_TEXT_ENGINE_FIDELITY_FIXTURES.md` | 4 | `7099fa49aca6e9574dc7ea5847914d8c20969222b5f2cf79090016539044e107` |
| `workspace_crates` | `specs/PROGRESS.md` | 8 | `d48e33427561b9af0d9986d51f36230e8321898487657f43a46f7c1c0e2ed4da` |
| `daemon_dispatch_methods` | `specs/PROGRESS.md` | 41 | `7c6fab120a23d8fbf495e171180071b7ce7ac0ef113898f8b97c3e51da00e5f6` |
| `engine_api_pub_fns` | `specs/ENGINE_SPEC.md` | 194 | `efc32d4daf16cfcc2997f9f9d85353d53aca4d768db9090dec2c6128b0aaedb4` |
| `standards_check_surface` | `specs/CHECKING_ARCHITECTURE_SPEC.md` | 29 | `56e6d1bca3d5e3245655ab9e4f5089013b0b1368156a4b7303aabd394550f2af` |
| `pool_library_surface` | `docs/contracts/LIBRARY_AUTHORING_TOOL_CONTRACT.md` | 114 | `edb4bf8faf38c8007792ce1e0c47760e18a87d2fdfb0a5db45e1b9ca844c22e6` |
