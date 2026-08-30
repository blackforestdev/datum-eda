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
| `mcp_runtime_methods` | `specs/MCP_API_SPEC.md` | 190 | `f9cd8102aff153cd4c9f3a86ecc501fc99aac66d41010eb8d33309740e5b48f5` |
| `cli_project_commands` | `specs/PROGRAM_SPEC.md` | 289 | `e6113b1c79f114e70e745ec2fd3bc73afc3cb4302d429ad67dfb2fff76ddbce1` |
| `engine_text_modules` | `docs/gui/DATUM_TEXT_ENGINE_PHASE_2_IMPLEMENTATION_PLAN.md` | 11 | `1233903bce862aa7ef22879e67e8cbef3bae2bf5e823bff9e53f39b4735c8059` |
| `m7_text_visual_fixtures` | `docs/gui/DATUM_TEXT_ENGINE_FIDELITY_FIXTURES.md` | 5 | `990b4a0e4fd6bddc8a5e74b6f2ee5795f32800c2dafcc40dd31476c1515d64c8` |
| `workspace_crates` | `specs/PROGRESS.md` | 10 | `8f610e4e2a644b1a60bbf735eca22e1193b711eb4e2bb93ad959a9a477c1279c` |
| `daemon_dispatch_methods` | `specs/PROGRESS.md` | 43 | `a1a7bd690633d6ebfbd765988cda081ce14c167bec3086e12a0648671f8131d7` |
| `engine_api_pub_fns` | `specs/ENGINE_SPEC.md` | 196 | `b5e294d1a0186d4c58b4214f54eda852b1df2ce5b41c2e22c204c361ab21cea8` |
| `standards_check_surface` | `specs/CHECKING_ARCHITECTURE_SPEC.md` | 29 | `56e6d1bca3d5e3245655ab9e4f5089013b0b1368156a4b7303aabd394550f2af` |
| `pool_library_surface` | `docs/contracts/LIBRARY_AUTHORING_TOOL_CONTRACT.md` | 115 | `2cbc95f1de4a410dbe6fb181eec5ffe4644afcc7ef946be8254c12fae893d1d2` |
| `erc_pin_taxonomy_surface` | `specs/ERC_SPEC.md` | 33 | `8f44622d66f182ef0d12cd5a49eb0033647eb56f1b277adc0645b5ae3033a8a9` |
| `schematic_connectivity_surface` | `specs/SCHEMATIC_CONNECTIVITY_SPEC.md` | 9 | `9e6f3473c2eea9b28598a7e8cf7b24c8b0fef6687ced07442e0bf9920f4e55ed` |
| `zone_fill_surface` | `specs/NATIVE_FORMAT_SPEC.md` | 19 | `8d10f280ffc6abcaf7990ce3120a4253a2ea54b474b0a0d56428b2e30bdd2dfa` |
| `gui_supervision_surface` | `specs/PROGRESS.md` | 9 | `bf469cb5d3ef2b1d74295a43cef0d3c52b3fc0e6d7961f0784dd2aa0c07132d3` |
| `source_health_debt_surface` | `docs/SOURCE_HEALTH_POLICY.md` | 94 | `8053e92adfa9c98f09fad9a5b82eb281dd0b11a37352d97a778c14c1c2278a15` |
