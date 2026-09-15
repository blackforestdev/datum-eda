#!/usr/bin/env python3
"""Fail if migrated state/evidence paths bypass their substrate-owned writers."""

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

SCHEMATIC_AUTHORING_FILES = [
    Path("crates/cli/src/commands/schematic/connectivity_mutations.rs"),
    Path("crates/cli/src/commands/schematic/symbol_mutations.rs"),
    Path("crates/cli/src/commands/schematic/text_drawing_mutations.rs"),
    Path("crates/cli/src/commands/schematic/sheet_mutations.rs"),
    Path("crates/cli/src/commands/schematic/proposals.rs"),
]

SCHEMATIC_HELPER_FILES = [
    Path("crates/cli/src/commands/schematic/helpers.rs"),
]

PRODUCTION_AUTHORING_FILES = [
    Path("crates/cli/src/commands/manufacturing/plans.rs"),
    Path("crates/cli/src/commands/output_jobs/output_jobs.rs"),
]

PROJECT_BOOTSTRAP_FILES = [
    Path("crates/cli/src/commands/project/roots.rs"),
]

ROUTE_STRATEGY_FIXTURE_FILES = [
    Path("crates/cli/src/commands/route/proposal.rs"),
]

LEGACY_KICAD_MODIFY_FILES = {
    Path("crates/cli/src/command_modify/modify_ops.rs"): {
        "save_calls": 1,
        "save_original_calls": 1,
        "required": [
            "const LEGACY_KICAD_MODIFY_SAVE_RETIRED_MESSAGE: &str",
            "legacy KiCad modify persistence is retired for production builds",
            "if (save.is_some() || save_original) && !legacy_kicad_modify_save_allowed()",
            "fn legacy_kicad_modify_save_allowed() -> bool {",
            "cfg!(test)",
        ],
    },
}

FORWARD_ANNOTATION_REVIEW_FILES = [
    Path("crates/cli/src/commands/forward_annotation/apply_review.rs"),
    Path("crates/cli/src/commands/forward_annotation/artifact_review.rs"),
]

PROPOSAL_APPLY_FILES = [
    Path("crates/cli/src/commands/forward_annotation/substrate.rs"),
    Path("crates/cli/src/commands/route/proposal_substrate.rs"),
]

LEGACY_PROPOSAL_SIDECAR_FILES = {
    Path("crates/engine/src/substrate/proposal.rs"): {
        "writes": 0,
        "required": [
            "pub fn commit_proposal_metadata_journaled(",
        ],
        "forbidden": [
            "pub fn write_proposal(",
            "pub fn write_legacy_proposal_sidecar(",
            "pub(super) fn write_legacy_proposal_sidecar(",
        ],
    },
}

SUBSTRATE_PUBLIC_EXPORT_FILES = {
    Path("crates/engine/src/substrate/mod.rs"): {
        "forbidden": [
            "write_legacy_import_map_sidecar",
            "write_legacy_proposal_sidecar",
            "persist_artifact_metadata",
            "persist_output_job_run",
            "persist_artifact_run",
            "persist_check_run",
            "persist_zone_fill",
        ],
    },
}

FORWARD_ANNOTATION_REVIEW_STATE_FILES = [
    Path("crates/cli/src/commands/forward_annotation/review_state.rs"),
]

GENERATED_EVIDENCE_FILES = {
    Path("crates/cli/src/commands/check/native_inspect.rs"): [
        "commit_check_run_evidence(",
        "build_set_check_run(",
        "commit_prepared(",
    ],
}

ENGINE_GENERATED_EVIDENCE_FILES = {
    Path("crates/engine/src/api/native_write/forward_annotation.rs"): {
        "writes": 0,
        "required": [
            'pub const FORWARD_ANNOTATION_REVIEW_PATH: &str = ".datum/forward_annotation_review/review.json";',
            "Operation::SetForwardAnnotationReview",
            "Operation::DeleteForwardAnnotationReview",
        ],
    },
    Path("crates/engine/src/api/native_write/genesis.rs"): {
        "writes": 0,
        "required": [
            "pub fn bootstrap_native_project(",
            "fn write_genesis_shard",
            "std::fs::rename(&temp_path, path)?;",
            "ProjectResolver::new(root).resolve()?;",
        ],
    },
    Path("crates/engine/src/substrate/generated_evidence.rs"): {
        "writes": 1,
        "required": [
            "pub(super) fn persist_generated_evidence",
            "std::fs::write(&temp_path, bytes.as_bytes())?;",
            "std::fs::rename(&temp_path, &path)?;",
            "sync_directory(&directory)?;",
        ],
    },
    Path("crates/engine/src/substrate/zone_fill.rs"): {
        "writes": 0,
        "required": [
            "#[allow(dead_code)]",
            "pub(super) fn persist_zone_fill(",
            'persist_generated_evidence(project_root, ".datum/zone_fills", &fill.zone_id, fill)',
        ],
    },
    Path("crates/engine/src/substrate/check_run.rs"): {
        "writes": 0,
        "required": [
            "#[allow(dead_code)]",
            "pub(super) fn persist_check_run(",
            'persist_generated_evidence(project_root, ".datum/check_runs", &run.check_run_id, run)',
        ],
    },
    Path("crates/engine/src/substrate/artifact_run.rs"): {
        "writes": 0,
        "required": [
            "#[allow(dead_code)]",
            "pub(super) fn persist_artifact_run(",
            'persist_generated_evidence(project_root, ".datum/artifact_runs", &run.run_id, run)',
        ],
    },
    Path("crates/engine/src/substrate/artifact.rs"): {
        "writes": 0,
        "required": [
            "#[allow(dead_code)]",
            "pub(super) fn persist_artifact_metadata(",
            'persist_generated_evidence(',
            '".datum/artifacts"',
            "pub(super) fn persist_output_job_run(",
            'persist_generated_evidence(project_root, ".datum/output_job_runs", &run.run_id, run)',
        ],
    },
}

ENGINE_SOURCE_STAGE_FILES = {
    Path("crates/engine/src/substrate/proposal_journal_ops.rs"): {
        "writes": 0,
        "required": [
            "pub(super) fn maybe_stage_proposal_operation(",
            "Operation::CreateProposalMetadata",
            "Operation::SetProposalMetadata",
            "SourceShardKind::ProposalMetadata",
            "stage_new_shard_write(",
        ],
    },
    Path("crates/engine/src/substrate/pool_journal_ops.rs"): {
        "writes": 0,
        "required": [
            "pub(super) fn maybe_stage_pool_operation(",
            "SourceShardKind::Pool",
            "stage_new_shard_write(",
        ],
    },
    Path("crates/engine/src/substrate/schematic_definition_journal_ops.rs"): {
        "writes": 0,
        "required": [
            "pub(super) fn maybe_stage_schematic_definition_operation(",
            "Operation::CreateSchematicDefinition",
            "SourceShardKind::SchematicDefinition",
            "stage_new_shard_write(",
        ],
    },
    Path("crates/engine/src/substrate/component_instance_journal_ops.rs"): {
        "writes": 0,
        "required": [
            "pub(super) fn stage_component_instance_operation(",
            "SourceShardKind::ComponentInstance",
            "stage_new_shard_write(",
        ],
    },
    Path("crates/engine/src/substrate/relationship_journal_ops.rs"): {
        "writes": 0,
        "required": [
            "pub(super) fn stage_relationship_operation(",
            "let relative_path = authored_relative_path(",
            "stage_new_shard_write(",
        ],
    },
    Path("crates/engine/src/substrate/production_journal_ops.rs"): {
        "writes": 0,
        "required": [
            "pub(super) fn stage_production_operation(",
            "let relative_path = production_relative_path(",
            "stage_new_shard_write(",
        ],
    },
    Path("crates/engine/src/substrate/zone_fill_journal_ops.rs"): {
        "writes": 0,
        "required": [
            "pub(super) fn maybe_stage_zone_fill_operation(",
            "Operation::SetZoneFill",
            "SourceShardKind::ZoneFill",
            "stage_new_shard_write(",
            'format!(".datum/zone_fills/{zone_id}.json")',
        ],
    },
    Path("crates/engine/src/substrate/import_map_journal_ops.rs"): {
        "writes": 0,
        "required": [
            "pub(super) fn maybe_stage_import_map_operation(",
            "Operation::CreateImportMapShard",
            "SourceShardKind::ImportMap",
            "stage_new_shard_write(",
        ],
    },
}

ENGINE_JOURNAL_PERSISTENCE_FILES = {
    Path("crates/engine/src/substrate/journal.rs"): {
        "writes": 3,
        "required": [
            "fn stage_shard_write(",
            "pub(super) fn stage_new_shard_write(",
            'join(".datum/stage")',
            "pub(super) fn promote_staged_shard_writes(",
            "std::fs::rename(staged, &write.destination)?;",
            "pub(super) fn write_journal_cursor(",
            "journal_cursor_path(project_root)",
            "pub(super) fn append_transaction_journal(",
            "transaction_journal_path(project_root)",
            "std::fs::OpenOptions::new()",
        ],
    },
}

GENERATED_EXPORT_FILES = {
    Path("crates/cli/src/commands/gerber/layers.rs"): {
        "writes": 5,
        "required": [
            "export_native_project_gerber_outline(",
            "export_native_project_gerber_copper_layer(",
            "export_native_project_gerber_soldermask_layer(",
            "export_native_project_gerber_silkscreen_layer(",
            "export_native_project_gerber_paste_layer(",
        ],
    },
    Path("crates/cli/src/commands/gerber/mechanical.rs"): {
        "writes": 1,
        "required": [
            "export_native_project_gerber_mechanical_layer(",
        ],
    },
    Path("crates/cli/src/commands/gerber/panel.rs"): {
        "writes": 1,
        "required": [
            "panelize_and_rewrite_rs274x_gerber_file(",
            "panelize_rs274x_gerber(",
        ],
    },
    Path("crates/cli/src/commands/drill/drill.rs"): {
        "writes": 2,
        "required": [
            "export_native_project_drill(",
            "export_native_project_excellon_drill(",
        ],
    },
    Path("crates/cli/src/commands/inventory/inventory.rs"): {
        "writes": 2,
        "required": [
            "export_native_project_bom(",
            "export_native_project_pnp(",
        ],
    },
}

GUI_APP_BOARD_TEXT_FILES = [
    Path("crates/gui-app/src/main.rs"),
]

RETIRED_GUI_BOARD_TEXT_FILES = [
    Path("crates/gui-protocol/src/board_text_field_values.rs"),
    Path("crates/gui-protocol/src/board_text_mutations.rs"),
]

FORBIDDEN_AUTHORING_PATTERNS = [
    "write_canonical_json(",
    "std::fs::write(",
]

FORBIDDEN_GENERATED_EVIDENCE_PATTERNS = [
    "write_artifact_metadata_manifest",
    "write_output_job_run_manifest",
    ".datum/artifacts",
    ".datum/artifact_runs",
    ".datum/output_job_runs",
    ".datum/zone_fills",
    ".datum/check_runs",
    "persist_check_run(",
    "write_canonical_json(",
    "std::fs::write(",
    "to_json_deterministic(",
]

FORBIDDEN_PRODUCTION_ZONE_FILL_PATTERNS = [
    "persist_zone_fill(",
]

REQUIRED_PRODUCTION_ZONE_FILL_PATTERNS = [
    "build_set_zone_fills(",
    "commit_prepared(",
]

ARTIFACT_ONLY_EVIDENCE_COMMAND_FILES = {
    Path("crates/cli/src/commands/artifacts/artifacts.rs"): {
        "required": [
            "commit_unlinked_artifact_evidence(",
            "commit_linked_artifact_output_job_evidence(",
        ],
        "forbidden": [
            "persist_artifact_metadata(",
            "persist_artifact_run(",
            "persist_output_job_run(",
        ],
    },
    Path("crates/cli/src/commands/artifacts/runs.rs"): {
        "required": [
            "pub(crate) fn generic_artifact_run(",
        ],
        "forbidden": [
            "persist_artifact_run(",
        ],
    },
    Path("crates/cli/src/commands/artifacts/drill.rs"): {
        "required": [
            "commit_unlinked_artifact_evidence(",
            "commit_linked_artifact_output_job_evidence(",
        ],
        "forbidden": [
            "persist_artifact_metadata(",
            "persist_artifact_run(",
            "persist_output_job_run(",
        ],
    },
    Path("crates/cli/src/commands/artifacts/output_runs.rs"): {
        "required": [
            "pub(super) fn generic_output_job_run(",
        ],
        "forbidden": [
            "persist_output_job_run(",
        ],
    },
    Path("crates/cli/src/commands/artifacts/validation.rs"): {
        "required": [
            "commit_artifact_metadata_evidence(",
        ],
        "forbidden": [
            "persist_artifact_metadata(",
        ],
    },
    Path("crates/cli/src/commands/artifacts/evidence.rs"): {
        "required": [
            "build_artifact_evidence(",
            "commit_prepared(",
        ],
        "forbidden": [
            "persist_artifact_metadata(",
            "persist_artifact_run(",
            "persist_output_job_run(",
        ],
    },
}

OUTPUT_JOB_RUN_EVIDENCE_COMMAND_FILES = {
    Path("crates/cli/src/commands/output_jobs/output_jobs.rs"): {
        "required": [
            "persist_output_job_run_journaled(",
        ],
        "forbidden": [
            "persist_output_job_run(",
        ],
    },
    Path("crates/cli/src/commands/output_jobs/runs.rs"): {
        "required": [
            "build_set_output_job_run(",
            "commit_prepared(",
            "pub(super) fn persist_output_job_run_journaled(",
        ],
        "forbidden": [
            "persist_output_job_run(",
        ],
    },
}

GERBER_EVIDENCE_COMMAND_FILES = {
    Path("crates/cli/src/commands/gerber/plan.rs"): {
        "required": [
            "commit_gerber_set_evidence(",
        ],
        "forbidden": [
            "persist_artifact_metadata(",
            "persist_output_job_run(",
        ],
    },
    Path("crates/cli/src/commands/gerber/evidence.rs"): {
        "required": [
            "build_artifact_evidence(",
            "commit_prepared(",
            "fn commit_gerber_set_evidence(",
        ],
        "forbidden": [
            "persist_artifact_metadata(",
            "persist_output_job_run(",
        ],
    },
}

MANUFACTURING_EVIDENCE_COMMAND_FILES = {
    Path("crates/cli/src/commands/manufacturing/manufacturing.rs"): {
        "required": [
            "commit_manufacturing_set_evidence(",
        ],
        "forbidden": [
            "persist_artifact_metadata(",
            "persist_output_job_run(",
        ],
    },
    Path("crates/cli/src/commands/manufacturing/scope.rs"): {
        "required": [
            "commit_manufacturing_set_evidence(",
        ],
        "forbidden": [
            "persist_artifact_metadata(",
        ],
    },
    Path("crates/cli/src/commands/manufacturing/evidence.rs"): {
        "required": [
            "build_artifact_evidence(",
            "commit_prepared(",
            "pub(crate) fn commit_manufacturing_set_evidence(",
        ],
        "forbidden": [
            "persist_artifact_metadata(",
            "persist_output_job_run(",
        ],
    },
}

FORBIDDEN_HELPER_PATTERNS = [
    "fn write_",
]

FORBIDDEN_FORWARD_ANNOTATION_STATE_PATTERNS = [
    "std::fs::write(",
    "write_canonical_json(",
    "project.json",
]

FORBIDDEN_PROPOSAL_APPLY_PATTERNS = [
    "write_proposal(",
    "model.proposals.insert(",
]

REQUIRED_PROPOSAL_APPLY_PATTERNS = [
    "commit_proposal_metadata_journaled(",
    "apply_accepted_proposal(",
]

REQUIRED_FORWARD_ANNOTATION_STATE_PATTERNS = [
    "pub(crate) fn write_forward_annotation_review(",
    "build_set_forward_annotation_review(",
    "build_clear_forward_annotation_review(",
    "commit_prepared(",
]

REQUIRED_PROJECT_BOOTSTRAP_PATTERNS = [
    "bootstrap_native_project(",
    "GenesisSpec {",
    "use eda_engine::api::native_write::genesis::",
]

REQUIRED_ROUTE_STRATEGY_FIXTURE_PATTERNS = [
    "write_route_strategy_batch_requests_manifest(",
    "route_proposal::build_route_strategy_fixture_board_write(",
    "route_proposal::build_route_strategy_fixture_net_class_clear(",
    "commit_prepared(&mut model, root, prepared)?;",
    "write_canonical_json(output_path, &artifact)?;",
    "write_canonical_json(&result_artifact_path, &report)?;",
]

FORBIDDEN_GUI_BOARD_TEXT_PATTERNS = [
    "toggle_board_text_boolean_field",
    "cycle_board_text_alignment_field",
    "cycle_board_text_field",
    "step_board_text_line_spacing_ratio",
    "step_board_text_height",
    "step_board_text_rotation",
    "set_board_text_alignment",
    "set_board_text_content",
    "set_board_text_font_family",
    "set_board_text_h_align",
    "set_board_text_height",
    "set_board_text_line_spacing_ratio",
    "set_board_text_render_intent",
    "set_board_text_rotation",
    "set_board_text_v_align",
]


def matching_lines(path: Path, patterns: list[str]) -> list[tuple[int, str, str]]:
    matches: list[tuple[int, str, str]] = []
    for index, line in enumerate(path.read_text().splitlines(), start=1):
        stripped = line.strip()
        for pattern in patterns:
            if pattern in stripped:
                matches.append((index, pattern, stripped))
    return matches


def count_pattern(path: Path, pattern: str) -> int:
    return path.read_text().count(pattern)


def require_patterns(
    failures: list[str], relative: Path, text: str, patterns: list[str], label: str
) -> None:
    for pattern in patterns:
        if pattern not in text:
            failures.append(f"{relative}: missing expected {label} pattern `{pattern}`")


def main() -> int:
    failures: list[str] = []

    for relative in SCHEMATIC_AUTHORING_FILES:
        path = ROOT / relative
        if not path.exists():
            failures.append(f"missing schematic authoring file: {relative}")
            continue
        for line, pattern, text in matching_lines(path, FORBIDDEN_AUTHORING_PATTERNS):
            failures.append(
                f"{relative}:{line}: forbidden private schematic writer `{pattern}`: {text}"
            )

    for relative in PRODUCTION_AUTHORING_FILES:
        path = ROOT / relative
        if not path.exists():
            failures.append(f"missing production authoring file: {relative}")
            continue
        for line, pattern, text in matching_lines(path, FORBIDDEN_AUTHORING_PATTERNS):
            failures.append(
                f"{relative}:{line}: forbidden private production writer `{pattern}`: {text}"
            )

    for relative in PROJECT_BOOTSTRAP_FILES:
        path = ROOT / relative
        if not path.exists():
            failures.append(f"missing project bootstrap file: {relative}")
            continue
        text = path.read_text()
        require_patterns(
            failures, relative, text, REQUIRED_PROJECT_BOOTSTRAP_PATTERNS, "project bootstrap"
        )
        write_count = count_pattern(path, "write_canonical_json(")
        if write_count != 0:
            failures.append(
                f"{relative}: expected 0 project bootstrap write_canonical_json calls (genesis is engine-owned), found {write_count}"
            )
        for line, pattern, text_line in matching_lines(path, ["std::fs::write("]):
            failures.append(
                f"{relative}:{line}: forbidden project bootstrap writer `{pattern}`: {text_line}"
            )

    for relative in ROUTE_STRATEGY_FIXTURE_FILES:
        path = ROOT / relative
        if not path.exists():
            failures.append(f"missing route-strategy fixture file: {relative}")
            continue
        text = path.read_text()
        require_patterns(
            failures,
            relative,
            text,
            REQUIRED_ROUTE_STRATEGY_FIXTURE_PATTERNS,
            "route-strategy fixture/artifact",
        )
        write_count = count_pattern(path, "write_canonical_json(")
        if write_count != 3:
            failures.append(
                f"{relative}: expected exactly 3 route-strategy artifact write_canonical_json calls (fixture boards are facade-committed), found {write_count}"
            )
        for line, pattern, text_line in matching_lines(path, ["std::fs::write("]):
            failures.append(
                f"{relative}:{line}: forbidden route-strategy fixture writer `{pattern}`: {text_line}"
            )

    for relative, config in LEGACY_KICAD_MODIFY_FILES.items():
        path = ROOT / relative
        if not path.exists():
            failures.append(f"missing legacy KiCad modify file: {relative}")
            continue
        text = path.read_text()
        require_patterns(
            failures,
            relative,
            text,
            list(config["required"]),
            "legacy KiCad modify retirement guard",
        )
        save_count = count_pattern(path, ".save(")
        expected_save_count = int(config["save_calls"])
        if save_count != expected_save_count:
            failures.append(
                f"{relative}: expected exactly {expected_save_count} legacy KiCad save call, found {save_count}"
            )
        save_original_count = count_pattern(path, ".save_to_original(")
        expected_save_original_count = int(config["save_original_calls"])
        if save_original_count != expected_save_original_count:
            failures.append(
                f"{relative}: expected exactly {expected_save_original_count} legacy KiCad save-original call, found {save_original_count}"
            )
        for line, pattern, text_line in matching_lines(
            path, ["write_canonical_json(", "std::fs::write("]
        ):
            failures.append(
                f"{relative}:{line}: forbidden legacy KiCad modify private writer `{pattern}`: {text_line}"
            )

    for relative in FORWARD_ANNOTATION_REVIEW_FILES:
        path = ROOT / relative
        if not path.exists():
            failures.append(f"missing forward-annotation review file: {relative}")
            continue
        for line, pattern, text in matching_lines(path, FORBIDDEN_AUTHORING_PATTERNS):
            failures.append(
                f"{relative}:{line}: forbidden private forward-annotation review writer `{pattern}`: {text}"
            )

    for relative in PROPOSAL_APPLY_FILES:
        path = ROOT / relative
        if not path.exists():
            failures.append(f"missing proposal apply file: {relative}")
            continue
        text = path.read_text()
        require_patterns(
            failures,
            relative,
            text,
            REQUIRED_PROPOSAL_APPLY_PATTERNS,
            "journaled proposal apply",
        )
        for line, pattern, text_line in matching_lines(path, FORBIDDEN_PROPOSAL_APPLY_PATTERNS):
            failures.append(
                f"{relative}:{line}: forbidden direct proposal sidecar/apply writer `{pattern}`: {text_line}"
            )

    for relative, config in LEGACY_PROPOSAL_SIDECAR_FILES.items():
        path = ROOT / relative
        if not path.exists():
            failures.append(f"missing legacy proposal sidecar file: {relative}")
            continue
        text = path.read_text()
        require_patterns(
            failures,
            relative,
            text,
            list(config["required"]),
            "legacy proposal sidecar boundary",
        )
        write_count = count_pattern(path, "std::fs::write(")
        expected_writes = int(config["writes"])
        if write_count != expected_writes:
            failures.append(
                f"{relative}: expected exactly {expected_writes} legacy proposal sidecar std::fs::write call, found {write_count}"
            )
        for line, pattern, text_line in matching_lines(path, list(config["forbidden"])):
            failures.append(
                f"{relative}:{line}: forbidden legacy proposal sidecar API `{pattern}`: {text_line}"
            )

    for relative, config in SUBSTRATE_PUBLIC_EXPORT_FILES.items():
        path = ROOT / relative
        if not path.exists():
            failures.append(f"missing substrate public export file: {relative}")
            continue
        for line, pattern, text_line in matching_lines(path, list(config["forbidden"])):
            failures.append(
                f"{relative}:{line}: forbidden public substrate legacy export `{pattern}`: {text_line}"
            )

    for relative in FORWARD_ANNOTATION_REVIEW_STATE_FILES:
        path = ROOT / relative
        if not path.exists():
            failures.append(f"missing forward-annotation review-state file: {relative}")
            continue
        text = path.read_text()
        require_patterns(
            failures,
            relative,
            text,
            REQUIRED_FORWARD_ANNOTATION_STATE_PATTERNS,
            "forward-annotation review-state",
        )
        for line, pattern, text_line in matching_lines(
            path, FORBIDDEN_FORWARD_ANNOTATION_STATE_PATTERNS
        ):
            failures.append(
                f"{relative}:{line}: forbidden forward-annotation review-state writer `{pattern}`: {text_line}"
            )

    for relative in SCHEMATIC_HELPER_FILES:
        path = ROOT / relative
        if not path.exists():
            failures.append(f"missing schematic helper file: {relative}")
            continue
        for line, pattern, text in matching_lines(path, FORBIDDEN_HELPER_PATTERNS):
            failures.append(
                f"{relative}:{line}: forbidden schematic writer helper `{pattern}`: {text}"
            )

    for relative, required_patterns in GENERATED_EVIDENCE_FILES.items():
        path = ROOT / relative
        if not path.exists():
            failures.append(f"missing generated-evidence file: {relative}")
            continue
        text = path.read_text()
        require_patterns(
            failures,
            relative,
            text,
            required_patterns,
            "generated-evidence substrate helper",
        )
        for line, pattern, text in matching_lines(path, FORBIDDEN_GENERATED_EVIDENCE_PATTERNS):
            failures.append(
                f"{relative}:{line}: forbidden generated-evidence writer `{pattern}`: {text}"
            )

    zone_fill_command = Path("crates/cli/src/commands/board/routing_net.rs")
    zone_fill_command_path = ROOT / zone_fill_command
    if not zone_fill_command_path.exists():
        failures.append(f"missing production ZoneFill command file: {zone_fill_command}")
    else:
        text = zone_fill_command_path.read_text()
        require_patterns(
            failures,
            zone_fill_command,
            text,
            REQUIRED_PRODUCTION_ZONE_FILL_PATTERNS,
            "journaled production ZoneFill command",
        )

    # rglob: scan every CLI source file regardless of directory layout so
    # file moves can never silently shrink the forbidden-writer scan scope.
    for path in sorted((ROOT / "crates/cli/src").rglob("*.rs")):
        relative = path.relative_to(ROOT)
        for line, pattern, text_line in matching_lines(
            path, FORBIDDEN_PRODUCTION_ZONE_FILL_PATTERNS
        ):
            failures.append(
                f"{relative}:{line}: forbidden CLI ZoneFill evidence writer `{pattern}`: {text_line}"
            )

    for relative, config in ARTIFACT_ONLY_EVIDENCE_COMMAND_FILES.items():
        path = ROOT / relative
        if not path.exists():
            failures.append(f"missing artifact-only evidence command file: {relative}")
            continue
        text = path.read_text()
        require_patterns(
            failures,
            relative,
            text,
            list(config["required"]),
            "journaled artifact-only generated evidence command",
        )
        for line, pattern, text_line in matching_lines(path, list(config["forbidden"])):
            failures.append(
                f"{relative}:{line}: forbidden direct artifact-only evidence writer `{pattern}`: {text_line}"
            )

    for relative, config in OUTPUT_JOB_RUN_EVIDENCE_COMMAND_FILES.items():
        path = ROOT / relative
        if not path.exists():
            failures.append(f"missing output-job-run evidence command file: {relative}")
            continue
        text = path.read_text()
        require_patterns(
            failures,
            relative,
            text,
            list(config["required"]),
            "journaled output-job-run generated evidence command",
        )
        for line, pattern, text_line in matching_lines(path, list(config["forbidden"])):
            failures.append(
                f"{relative}:{line}: forbidden direct output-job-run evidence writer `{pattern}`: {text_line}"
            )

    for relative, config in GERBER_EVIDENCE_COMMAND_FILES.items():
        path = ROOT / relative
        if not path.exists():
            failures.append(f"missing Gerber evidence command file: {relative}")
            continue
        text = path.read_text()
        require_patterns(
            failures,
            relative,
            text,
            list(config["required"]),
            "journaled Gerber generated evidence command",
        )
        for line, pattern, text_line in matching_lines(path, list(config["forbidden"])):
            failures.append(
                f"{relative}:{line}: forbidden direct Gerber evidence writer `{pattern}`: {text_line}"
            )

    for relative, config in MANUFACTURING_EVIDENCE_COMMAND_FILES.items():
        path = ROOT / relative
        if not path.exists():
            failures.append(f"missing manufacturing evidence command file: {relative}")
            continue
        text = path.read_text()
        require_patterns(
            failures,
            relative,
            text,
            list(config["required"]),
            "journaled manufacturing generated evidence command",
        )
        for line, pattern, text_line in matching_lines(path, list(config["forbidden"])):
            failures.append(
                f"{relative}:{line}: forbidden direct manufacturing evidence writer `{pattern}`: {text_line}"
            )

    for relative, config in ENGINE_GENERATED_EVIDENCE_FILES.items():
        path = ROOT / relative
        if not path.exists():
            failures.append(f"missing engine generated-evidence helper file: {relative}")
            continue
        text = path.read_text()
        require_patterns(
            failures,
            relative,
            text,
            list(config["required"]),
            "engine generated-evidence helper",
        )
        write_count = count_pattern(path, "std::fs::write(")
        expected_writes = int(config["writes"])
        if write_count != expected_writes:
            failures.append(
                f"{relative}: expected exactly {expected_writes} engine generated-evidence std::fs::write calls, found {write_count}"
            )
        for line, pattern, text_line in matching_lines(path, ["write_canonical_json("]):
            failures.append(
                f"{relative}:{line}: forbidden engine generated-evidence canonical JSON writer `{pattern}`: {text_line}"
            )

    for relative, config in ENGINE_SOURCE_STAGE_FILES.items():
        path = ROOT / relative
        if not path.exists():
            failures.append(f"missing engine source-stage file: {relative}")
            continue
        text = path.read_text()
        require_patterns(
            failures,
            relative,
            text,
            list(config["required"]),
            "engine source-stage helper",
        )
        write_count = count_pattern(path, "std::fs::write(")
        expected_writes = int(config["writes"])
        if write_count != expected_writes:
            failures.append(
                f"{relative}: expected exactly {expected_writes} engine source-stage std::fs::write calls, found {write_count}"
            )
        for line, pattern, text_line in matching_lines(path, ["write_canonical_json("]):
            failures.append(
                f"{relative}:{line}: forbidden engine source-stage canonical JSON writer `{pattern}`: {text_line}"
            )

    for relative, config in ENGINE_JOURNAL_PERSISTENCE_FILES.items():
        path = ROOT / relative
        if not path.exists():
            failures.append(f"missing engine journal persistence file: {relative}")
            continue
        text = path.read_text()
        require_patterns(
            failures,
            relative,
            text,
            list(config["required"]),
            "engine journal persistence owner",
        )
        write_count = count_pattern(path, "std::fs::write(")
        expected_writes = int(config["writes"])
        if write_count != expected_writes:
            failures.append(
                f"{relative}: expected exactly {expected_writes} engine journal-persistence std::fs::write calls, found {write_count}"
            )

    for relative, config in GENERATED_EXPORT_FILES.items():
        path = ROOT / relative
        if not path.exists():
            failures.append(f"missing generated-export file: {relative}")
            continue
        text = path.read_text()
        require_patterns(
            failures,
            relative,
            text,
            list(config["required"]),
            "generated-export",
        )
        write_count = count_pattern(path, "std::fs::write(")
        expected_writes = int(config["writes"])
        if write_count != expected_writes:
            failures.append(
                f"{relative}: expected exactly {expected_writes} generated-export std::fs::write calls, found {write_count}"
            )
        for line, pattern, text_line in matching_lines(path, ["write_canonical_json("]):
            failures.append(
                f"{relative}:{line}: forbidden generated-export canonical JSON writer `{pattern}`: {text_line}"
            )

    for relative in GUI_APP_BOARD_TEXT_FILES:
        path = ROOT / relative
        if not path.exists():
            failures.append(f"missing GUI board-text file: {relative}")
            continue
        for line, pattern, text in matching_lines(path, FORBIDDEN_GUI_BOARD_TEXT_PATTERNS):
            failures.append(
                f"{relative}:{line}: forbidden GUI board-text private writer `{pattern}`: {text}"
            )

    for relative in RETIRED_GUI_BOARD_TEXT_FILES:
        path = ROOT / relative
        if path.exists():
            failures.append(f"{relative}: retired GUI board-text private writer file reappeared")

    if failures:
        print("Private-writer guard failed:")
        for failure in failures:
            print(f"  - {failure}")
        print(
            "Migrated authoring paths must commit typed OperationBatch entries "
            "instead of mutating authored JSON directly; generated evidence "
            "manifests must use substrate persistence helpers; GUI board-text "
            "edits must route through terminal-prefilled journaled CLI commands; "
            "forward-annotation review state must remain confined to its explicit sidecar; "
            "project bootstrap and route-strategy fixture writes must stay within their "
            "classified bootstrap/generated boundaries; generated export writers must stay "
            "within exact-count file-output surfaces."
        )
        return 1

    print(
        "Private-writer guard passed "
        f"({len(SCHEMATIC_AUTHORING_FILES)} schematic authoring files, "
        f"{len(PRODUCTION_AUTHORING_FILES)} production authoring files, "
        f"{len(PROJECT_BOOTSTRAP_FILES)} project bootstrap files, "
        f"{len(ROUTE_STRATEGY_FIXTURE_FILES)} route-strategy fixture files, "
        f"{len(LEGACY_KICAD_MODIFY_FILES)} legacy KiCad modify files, "
        f"{len(FORWARD_ANNOTATION_REVIEW_FILES)} forward-annotation review files, "
        f"{len(PROPOSAL_APPLY_FILES)} proposal apply files, "
        f"{len(LEGACY_PROPOSAL_SIDECAR_FILES)} legacy proposal sidecar files, "
        f"{len(GENERATED_EVIDENCE_FILES)} generated-evidence files, "
        "1 production ZoneFill command family, "
        f"{len(ARTIFACT_ONLY_EVIDENCE_COMMAND_FILES)} artifact-only evidence command files, "
        f"{len(OUTPUT_JOB_RUN_EVIDENCE_COMMAND_FILES)} output-job-run evidence command files, "
        f"{len(GERBER_EVIDENCE_COMMAND_FILES)} Gerber evidence command files, "
        f"{len(MANUFACTURING_EVIDENCE_COMMAND_FILES)} manufacturing evidence command files, "
        f"{len(ENGINE_GENERATED_EVIDENCE_FILES)} engine generated-evidence files, "
        f"{len(ENGINE_SOURCE_STAGE_FILES)} engine source-stage files, "
        f"{len(ENGINE_JOURNAL_PERSISTENCE_FILES)} engine journal-persistence files, "
        f"{len(GENERATED_EXPORT_FILES)} generated-export files, "
        f"{len(GUI_APP_BOARD_TEXT_FILES)} GUI board-text files, "
        f"{len(FORWARD_ANNOTATION_REVIEW_STATE_FILES)} retired forward-annotation review-state paths, "
        f"{len(RETIRED_GUI_BOARD_TEXT_FILES)} retired GUI board-text paths)."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
