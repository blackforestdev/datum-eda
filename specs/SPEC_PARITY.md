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
| `workflow_delivery_contract_instances` | `specs/WORKFLOW_DELIVERY_GATE_CONTRACT.md` | 6 | `a6a7114d0266bae6408f57949cf0ed65c317fcc8a91b71433d1f34a9916d4df5` |
| `workflow_delivery_environment_selections` | `docs/decisions/PRODUCT_MECHANICS_042_BROAD_WORKFLOW_DELIVERY_ENFORCEMENT.md` | 2 | `52776ee3874773fe795616e958a8d420def65bd5aeb622c7924707316ea92252` |
| `mcp_runtime_methods` | `specs/MCP_API_SPEC.md` | 190 | `f9cd8102aff153cd4c9f3a86ecc501fc99aac66d41010eb8d33309740e5b48f5` |
| `cli_project_commands` | `specs/PROGRAM_SPEC.md` | 289 | `e6113b1c79f114e70e745ec2fd3bc73afc3cb4302d429ad67dfb2fff76ddbce1` |
| `engine_text_modules` | `docs/gui/DATUM_TEXT_ENGINE_PHASE_2_IMPLEMENTATION_PLAN.md` | 11 | `1233903bce862aa7ef22879e67e8cbef3bae2bf5e823bff9e53f39b4735c8059` |
| `m7_text_visual_fixtures` | `docs/gui/DATUM_TEXT_ENGINE_FIDELITY_FIXTURES.md` | 5 | `990b4a0e4fd6bddc8a5e74b6f2ee5795f32800c2dafcc40dd31476c1515d64c8` |
| `workspace_crates` | `specs/PROGRESS.md` | 10 | `8f610e4e2a644b1a60bbf735eca22e1193b711eb4e2bb93ad959a9a477c1279c` |
| `daemon_dispatch_methods` | `specs/PROGRESS.md` | 43 | `a1a7bd690633d6ebfbd765988cda081ce14c167bec3086e12a0648671f8131d7` |
| `engine_api_pub_fns` | `specs/ENGINE_SPEC.md` | 210 | `0160faf8c9120f9fda738007805438b851cbd99013bfd9994767af0229c8b78a` |
| `standards_check_surface` | `specs/CHECKING_ARCHITECTURE_SPEC.md` | 29 | `56e6d1bca3d5e3245655ab9e4f5089013b0b1368156a4b7303aabd394550f2af` |
| `pool_library_surface` | `docs/contracts/LIBRARY_AUTHORING_TOOL_CONTRACT.md` | 115 | `2cbc95f1de4a410dbe6fb181eec5ffe4644afcc7ef946be8254c12fae893d1d2` |
| `erc_pin_taxonomy_surface` | `specs/ERC_SPEC.md` | 33 | `8f44622d66f182ef0d12cd5a49eb0033647eb56f1b277adc0645b5ae3033a8a9` |
| `schematic_connectivity_surface` | `specs/SCHEMATIC_CONNECTIVITY_SPEC.md` | 9 | `9e6f3473c2eea9b28598a7e8cf7b24c8b0fef6687ced07442e0bf9920f4e55ed` |
| `zone_fill_surface` | `specs/NATIVE_FORMAT_SPEC.md` | 19 | `0d9d4f46a6326aa551810d2f47ec11ada1fb4ed11712455d07d7320a1fa1ac84` |
| `gui_supervision_surface` | `specs/PROGRESS.md` | 9 | `bf469cb5d3ef2b1d74295a43cef0d3c52b3fc0e6d7961f0784dd2aa0c07132d3` |
| `source_health_debt_surface` | `docs/SOURCE_HEALTH_POLICY.md` | 76 | `561e1d77c21cc9aec44ee755f3d76aabb1328581b1682b949eb03f9d12e66284` |
| `global_preferences_product_surface` | `specs/GLOBAL_PREFERENCES_PRODUCT_SURFACE_CONTRACT.md` | 227 | `92f922ba4dce0e737bd7b702ba2899cfccd4a8a707e620605706469ac06ef30e` |
| `global_preferences_acceptance_contract` | `specs/GLOBAL_PREFERENCES_PRODUCTION_ACCEPTANCE_CONTRACT.md` | 1 | `84ef7b2bbe0f4dac20c00683b990ee74c078978b0c4339772beec6e1dea1f68d` |
| `gui_performance_adoption_contract` | `specs/GUI_PERFORMANCE_IMPLEMENTATION_CONTRACT.md` | 2 | `29e36b6edfcb15a3c866db0b0b241ee8a55bec9b49eaacdce2e5d4ad5b36e22e` |
| `gui_performance_acceptance_draft` | `specs/GUI_PERFORMANCE_RECOVERY_PLAN.md` | 1 | `94558a23cd566aff4133851ec4fe17accca4d85e26eca0666844398cc15bf6fc` |

The PM041 delivery inventory tracks the pilot and prepared rollout infrastructure
contract-instance filenames. Its registered shapes are not validated by this
file-glob inventory; closed-shape and behavioral refusal proof belong to the
delivery validator. The infrastructure instance is specified in
`specs/WORKFLOW_DELIVERY_INFRASTRUCTURE_CONTRACT.md` under WDQ-I02.
These names and counts do not establish readiness, enrollment, proof or activation.

| `electrical_net_anchor_reasons` | `specs/NATIVE_FORMAT_SPEC.md` | 4 | `8bef70b763921f01889d515d5528bcc216eb758f6ccee1c12822631c9b621aae` |
| `electrical_identity_kinds` | `specs/NATIVE_FORMAT_SPEC.md` | 4 | `1bf0f7c8dcc77c45ab259d2a3f6a1ae75c1e4d91bd9152b63c3948621ccc49b5` |
| `electrical_net_intents` | `specs/NATIVE_FORMAT_SPEC.md` | 5 | `1d88a30b5bcb2f5d61e11ff8fef2bf1d96774ce948aeccf114d69a513887b501` |
| `electrical_bus_interfaces` | `specs/NATIVE_FORMAT_SPEC.md` | 2 | `d50413aabab0fca6591550379f84402ac6cbdceade981d4bde7a82777ba06ecc` |
| `electrical_source_shard_kinds` | `specs/NATIVE_FORMAT_SPEC.md` | 22 | `4f2b2b1770e9f5d61d0e1e2ad3af9edde7fcc61f334a2fe07119b0d3d7bcef8b` |
| `electrical_source_shard_taxa` | `specs/NATIVE_FORMAT_SPEC.md` | 23 | `1ff1a8ca056dd0775614cf993bda5db903ebe046397c86f3a9d07995085eee27` |
| `canonical_operation_kinds` | `specs/NATIVE_FORMAT_SPEC.md` | 139 | `eb3b9d53279b79a33cf11d28e9d3d109d84c328baccbea176f966d0258b52a04` |
| `electrical_identity_source_schema` | `specs/NATIVE_FORMAT_SPEC.md` | 1 | `c60c0a32a9e73c4947f5b3dbff1f178c5dc9f1c81a1a8f5ac45a42aea666a1a9` |
