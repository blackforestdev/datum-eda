#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cargo_guard=(python3 "$repo_root/scripts/run_cargo_guarded.py" --workload proof --)
cd "$repo_root"

run_pg() {
  local gate="$1"
  shift
  printf '\n[PG] %s\n' "$gate"
  "$@"
}

run_pg "PG-IDENTITY-SUBSTRATE" \
  "${cargo_guard[@]}" cargo test -p datum-eda-cli main_tests_project_imports -- --nocapture

run_pg "PG-IDENTITY-SUBSTRATE:SCHEMATIC" \
  "${cargo_guard[@]}" cargo test -p datum-eda-cli main_tests_project_imports_schematic -- --nocapture

run_pg "PG-RESOLVER-RECOVERY" \
  "${cargo_guard[@]}" cargo test -p datum-eda-cli main_tests_project_source_shard -- --nocapture

run_pg "PG-SOURCE-SHARD-CONTEXT" \
  "${cargo_guard[@]}" cargo test -p datum-eda-cli main_tests_context_source_shards -- --nocapture

run_pg "PG-SOURCE-SHARD-GUI-SUMMARY" \
  "${cargo_guard[@]}" cargo test -p datum-gui-protocol source_shard_status -- --nocapture

run_pg "PG-COMMIT-ATOMIC+DURABLE-UNDO" \
  "${cargo_guard[@]}" cargo test -p eda-engine substrate::tests::journal_hardening -- --nocapture

run_pg "PG-REVISION-GUARDS" \
  "${cargo_guard[@]}" cargo test -p eda-engine substrate::tests::journal_revision_guards -- --nocapture

run_pg "PG-SHARD-DIFF-ISOLATION" \
  "${cargo_guard[@]}" cargo test -p datum-eda-cli project_query_relationships_and_variants_are_resolver_backed -- --nocapture

run_pg "PG-COMPONENT-INSTANCE-SUBSTRATE" \
  "${cargo_guard[@]}" cargo test -p eda-engine substrate::tests::component_instance -- --nocapture

run_pg "PG-COMPONENT-INSTANCE-PROPOSAL-POLICY" \
  "${cargo_guard[@]}" cargo test -p eda-engine substrate::tests::component_instance_proposal_policy -- --nocapture

run_pg "PG-LIBRARY-AUTHORING-SUBSTRATE" \
  "${cargo_guard[@]}" cargo test -p eda-engine substrate::tests::pool_library -- --nocapture

run_pg "PG-LIBRARY-AUTHORING-CLI" \
  "${cargo_guard[@]}" cargo test -p datum-eda-cli main_tests_project_pool_library -- --nocapture

run_pg "PG-LIBRARY-AUTHORING-PROPOSALS" \
  "${cargo_guard[@]}" cargo test -p datum-eda-cli main_tests_project_pool_package_proposals -- --nocapture

run_pg "PG-PROPOSAL-PARITY" \
  "${cargo_guard[@]}" cargo test -p eda-engine proposal_predicted_transaction_id_matches_preview_and_apply_transaction -- --nocapture

run_pg "PG-PROPOSAL-SCHEMA+REPLAY" \
  "${cargo_guard[@]}" cargo test -p eda-engine substrate::tests::proposal -- --nocapture

run_pg "PG-CHECKRUN-GENERATED-EVIDENCE" \
  "${cargo_guard[@]}" cargo test -p eda-engine substrate::tests::check_run -- --nocapture

run_pg "PG-CHECKRUN-CLI-LIFECYCLE" \
  "${cargo_guard[@]}" cargo test -p datum-eda-cli main_tests_project_check_run -- --nocapture

run_pg "PG-LIVE-CAM-EQUIVALENCE:GERBER-COPPER" \
  "${cargo_guard[@]}" cargo test -p datum-eda-cli project_validate_gerber_copper_layer_reports_match_and_mismatch -- --nocapture

run_pg "PG-LIVE-CAM-EQUIVALENCE:EXCELLON-DRILL" \
  "${cargo_guard[@]}" cargo test -p datum-eda-cli project_export_excellon_drill_writes_narrow_excellon_from_board_vias -- --nocapture

run_pg "PG-ZONEFILL-SUBSTRATE" \
  "${cargo_guard[@]}" cargo test -p eda-engine substrate::tests::zone_fill -- --nocapture

run_pg "PG-ZONEFILL-CLI-SOLVER" \
  "${cargo_guard[@]}" cargo test -p datum-eda-cli main_tests_project_board_zone_fill_solver -- --nocapture

run_pg "PG-PANELIZATION-ISOLATION" \
  "${cargo_guard[@]}" cargo test -p datum-eda-cli project_update_panel_projection_as_proposal_defers_mutation_until_apply -- --nocapture

run_pg "PG-PRODUCTION-SCHEMA+POLICY" \
  "${cargo_guard[@]}" cargo test -p eda-engine substrate::tests::production -- --nocapture

run_pg "PG-VARIANT-RESOLUTION" \
  "${cargo_guard[@]}" cargo test -p datum-eda-cli project_bom_and_pnp_variant_filter_uses_component_instance_population -- --nocapture

run_pg "PG-ARTIFACT-TRACEABILITY:OUTPUT-GRAPH" \
  "${cargo_guard[@]}" cargo test -p eda-engine generated_output_artifact_graph_replays_without_model_revision_mutation -- --nocapture

run_pg "PG-OUTPUT-JOB-RUN-REPLAY" \
  "${cargo_guard[@]}" cargo test -p eda-engine substrate::tests::output_job_run_replay -- --nocapture

run_pg "PG-GENERATED-EVIDENCE-COMMIT-SCOPE" \
  "${cargo_guard[@]}" cargo test -p eda-engine substrate::tests::generated_evidence_scope -- --nocapture

run_pg "PG-OUTPUT-JOB-CLI-REPLAY" \
  "${cargo_guard[@]}" cargo test -p datum-eda-cli main_tests_project_output_job_run_replay -- --nocapture

run_pg "PG-ARTIFACT-TRACEABILITY:MANUFACTURING-PROJECTION" \
  "${cargo_guard[@]}" cargo test -p datum-eda-cli manufacturing_export_files_match_manifest_projection -- --nocapture

run_pg "PG-STANDARDS-REPAIR-PROPOSALS" \
  "${cargo_guard[@]}" cargo test -p datum-eda-cli main_tests_project_standards_repair_apply -- --nocapture

run_pg "PG-STANDARDS-REPAIR-PROPOSALS:CLEARANCE" \
  "${cargo_guard[@]}" cargo test -p datum-eda-cli main_tests_project_standards_clearance_repairs -- --nocapture

run_pg "PG-STANDARDS-REPAIR-PROPOSALS:SILK" \
  "${cargo_guard[@]}" cargo test -p datum-eda-cli main_tests_project_standards_silk_repairs -- --nocapture

run_pg "PG-STANDARDS-REPAIR-PROPOSALS:PEER-APERTURE" \
  "${cargo_guard[@]}" cargo test -p datum-eda-cli main_tests_project_standards_peer_aperture_repairs -- --nocapture

run_pg "PG-STANDARDS-REPAIR-PROPOSALS:ZONE-FILL" \
  "${cargo_guard[@]}" cargo test -p datum-eda-cli main_tests_project_standards_zone_fill_repairs -- --nocapture

run_pg "PG-RESOLVER-BACKED-VALIDATION" \
  "${cargo_guard[@]}" cargo test -p datum-eda-cli main_tests_project_validate_resolver -- --nocapture

run_pg "PG-UI-LAYOUT-INVARIANTS" \
  "${cargo_guard[@]}" cargo test -p datum-gui-render layout_invariant_tests -- --nocapture
