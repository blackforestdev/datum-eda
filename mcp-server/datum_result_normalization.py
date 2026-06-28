#!/usr/bin/env python3
"""Canonical datum.* result payload normalization."""

from __future__ import annotations

from typing import Any


def normalize_datum_result(name: str, raw: Any) -> Any:
    if not isinstance(raw, dict):
        return raw
    if name.startswith("datum.check."):
        return _normalize_check_result(name, raw)
    if name.startswith("datum.artifact."):
        return _normalize_artifact_result(name, raw)
    if name.startswith("datum.proposal."):
        return _normalize_proposal_result(name, raw)
    if name.startswith("datum.journal."):
        return _normalize_journal_result(name, raw)
    if name.startswith("datum.component_instance."):
        return _normalize_component_instance_result(name, raw)
    if name.startswith("datum.query."):
        return _normalize_query_result(name, raw)
    return raw


def _normalize_check_result(name: str, raw: dict[str, Any]) -> dict[str, Any]:
    if name == "datum.check.run":
        return {
            "check_run_id": raw.get("check_run_id"),
            "persisted": raw.get("persisted"),
            "profile_id": raw.get("profile_id"),
            "status": raw.get("status"),
            "summary": raw.get("summary"),
            "finding_count": raw.get("finding_count", len(raw.get("findings", []))),
            "findings": [_normalize_finding(finding) for finding in raw.get("findings", [])],
            "proposal_refs": raw.get("proposal_refs", []),
            "proposal_links": raw.get("proposal_links", []),
            "profile_basis": raw.get("profile_basis", {}),
            "coverage": raw.get("coverage", []),
            "raw": raw,
        }
    if name == "datum.check.list":
        return {
            "check_run_count": raw.get("check_run_count", len(raw.get("check_runs", []))),
            "check_runs": raw.get("check_runs", []),
            "raw": raw,
        }
    if name == "datum.check.show":
        check_run = raw.get("check_run") if isinstance(raw.get("check_run"), dict) else {}
        return {
            "check_run_id": check_run.get("check_run_id"),
            "profile_id": check_run.get("profile_id"),
            "status": check_run.get("status"),
            "finding_count": check_run.get("finding_count", len(check_run.get("findings", []))),
            "findings": [_normalize_finding(finding) for finding in check_run.get("findings", [])],
            "proposal_refs": check_run.get("proposal_refs", []),
            "proposal_links": check_run.get("proposal_links", []),
            "profile_basis": check_run.get("profile_basis", {}),
            "coverage": check_run.get("coverage", []),
            "check_run": check_run,
            "raw": raw,
        }
    if name == "datum.check.profiles":
        return {
            "default_profile_id": raw.get("default_profile_id"),
            "profile_count": raw.get("profile_count", len(raw.get("profiles", []))),
            "profiles": raw.get("profiles", []),
            "raw": raw,
        }
    if name == "datum.check.repair_standards":
        proposals = raw.get("proposals", [])
        return {
            "check_run_id": raw.get("check_run_id"),
            "proposal_count": raw.get("proposal_count", len(proposals)),
            "proposals": [
                _normalize_standards_repair_proposal(proposal)
                for proposal in proposals
            ],
            "raw": raw,
        }
    return raw


def _normalize_finding(raw: Any) -> Any:
    if not isinstance(raw, dict):
        return raw
    return {
        "finding_id": raw.get("finding_id") or raw.get("id"),
        "fingerprint": raw.get("fingerprint"),
        "domain": raw.get("domain") or raw.get("source"),
        "rule_id": raw.get("rule_id") or raw.get("code"),
        "standards_basis": raw.get("standards_basis"),
        "rule_revision": raw.get("rule_revision"),
        "import_key": raw.get("import_key"),
        "severity": raw.get("severity"),
        "status": raw.get("status"),
        "primary_target": raw.get("primary_target"),
        "related_targets": raw.get("related_targets", []),
        "message": raw.get("message"),
        "explanation": raw.get("explanation"),
        "suggested_next_action": raw.get("suggested_next_action"),
        "evidence": raw.get("evidence", []),
        "proposal_refs": raw.get("proposal_refs", []),
        "proposal_links": raw.get("proposal_links", []),
        "waiver_refs": raw.get("waiver_refs", []),
        "deviation_refs": raw.get("deviation_refs", []),
        "raw": raw,
    }


def _normalize_standards_repair_proposal(raw: Any) -> Any:
    if not isinstance(raw, dict):
        return raw
    return {
        "proposal_id": raw.get("proposal_id") or raw.get("id"),
        "repair_kind": raw.get("repair_kind"),
        "affected_pad": raw.get("affected_pad"),
        "affected_track": raw.get("affected_track"),
        "affected_via": raw.get("affected_via"),
        "affected_net_class": raw.get("affected_net_class"),
        "affected_zone": raw.get("affected_zone"),
        "finding_fingerprints": raw.get("finding_fingerprints", []),
        "codes": raw.get("codes", []),
        "prepared_against": raw.get("prepared_against"),
        "prepared_against_current_model": raw.get("prepared_against_current_model"),
        "can_apply": raw.get("can_apply"),
        "blocker_codes": raw.get("blocker_codes", []),
        "operations": raw.get("operations"),
        "raw": raw,
    }


def _normalize_artifact_result(name: str, raw: dict[str, Any]) -> dict[str, Any]:
    if name == "datum.artifact.generate":
        return {
            "generated_count": raw.get("generated_count", len(raw.get("generated", []))),
            "generated": [
                _normalize_generated_artifact(generated)
                for generated in raw.get("generated", [])
            ],
            "include": raw.get("include", []),
            "output_dir": raw.get("output_dir"),
            "raw": raw,
        }
    if name == "datum.artifact.list":
        return {
            "artifact_count": raw.get("artifact_count", len(raw.get("artifacts", []))),
            "artifacts": [_normalize_artifact(artifact) for artifact in raw.get("artifacts", [])],
            "artifact_run_count": raw.get("artifact_run_count", len(raw.get("artifact_runs", []))),
            "artifact_runs": raw.get("artifact_runs", []),
            "raw": raw,
        }
    if name == "datum.artifact.show":
        return {
            "artifact": _normalize_artifact(raw.get("artifact")),
            "run_count": raw.get("run_count"),
            "latest_run": raw.get("latest_run"),
            "runs": raw.get("runs", []),
            "raw": raw,
        }
    if name == "datum.artifact.files":
        return {
            "artifact_id": raw.get("artifact_id"),
            "kind": raw.get("kind"),
            "output_dir": raw.get("output_dir"),
            "validation_state": raw.get("validation_state"),
            "file_count": raw.get("file_count", len(raw.get("files", []))),
            "files": [_normalize_file(file) for file in raw.get("files", [])],
            "production_projections": raw.get("production_projections", []),
            "raw": raw,
        }
    if name == "datum.artifact.preview":
        return {
            "artifact_id": raw.get("artifact_id"),
            "file": raw.get("file"),
            "file_path": raw.get("file_path"),
            "hash_matches_metadata": raw.get("hash_matches_metadata"),
            "preview_kind": raw.get("preview_kind"),
            "preview_available": raw.get("preview_available"),
            "inspection": raw.get("inspection"),
            "raw": raw,
        }
    if name == "datum.artifact.compare":
        return {
            "before_artifact_id": raw.get("before_artifact_id"),
            "after_artifact_id": raw.get("after_artifact_id"),
            "equivalent": raw.get("equivalent"),
            "files_equal": raw.get("files_equal"),
            "raw": raw,
        }
    if name in {"datum.artifact.validate", "datum.artifact.validate_manufacturing_set"}:
        return {
            "artifact_id": raw.get("artifact_id"),
            "valid": raw.get("valid"),
            "validation_state": raw.get("validation_state") or raw.get("artifact_validation_state"),
            "mismatch_count": raw.get("artifact_file_hash_mismatch_count"),
            "manifest_path": raw.get("artifact_manifest_path"),
            "raw": raw,
        }
    if name == "datum.artifact.export_manufacturing_set":
        return {
            "artifact": _normalize_artifact(raw.get("artifact_metadata")),
            "manifest_path": raw.get("artifact_manifest_path"),
            "output_job_run": raw.get("output_job_run"),
            "output_dir": raw.get("output_dir"),
            "raw": raw,
        }
    return raw


def _normalize_artifact(raw: Any) -> Any:
    if not isinstance(raw, dict):
        return raw
    return {
        "artifact_id": raw.get("artifact_id"),
        "kind": raw.get("kind"),
        "project_id": raw.get("project_id"),
        "model_revision": raw.get("model_revision"),
        "output_job": raw.get("output_job"),
        "variant": raw.get("variant"),
        "output_context": raw.get("output_context"),
        "generator_version": raw.get("generator_version"),
        "output_dir": raw.get("output_dir"),
        "validation_state": raw.get("validation_state"),
        "file_count": raw.get("file_count", len(raw.get("files", []))),
        "files": [_normalize_file(file) for file in raw.get("files", [])],
        "production_projections": raw.get("production_projections", []),
        "raw": raw,
    }


def _normalize_generated_artifact(raw: Any) -> Any:
    if not isinstance(raw, dict):
        return raw
    report = raw.get("report") if isinstance(raw.get("report"), dict) else {}
    artifact_metadata = report.get("artifact_metadata")
    if artifact_metadata is None:
        artifact_metadata = raw.get("artifact_metadata")
    return {
        "include": raw.get("include"),
        "artifact_id": raw.get("artifact_id"),
        "kind": raw.get("kind"),
        "model_revision": raw.get("model_revision"),
        "file_count": raw.get("file_count"),
        "artifact_manifest_path": raw.get("artifact_manifest_path"),
        "artifact": _normalize_artifact(artifact_metadata),
        "report": report,
        "raw": raw,
    }


def _normalize_file(raw: Any) -> Any:
    if not isinstance(raw, dict):
        return raw
    return {"path": raw.get("path"), "sha256": raw.get("sha256"), "raw": raw}


def _normalize_proposal_result(name: str, raw: dict[str, Any]) -> dict[str, Any]:
    if name == "datum.proposal.list":
        proposals = _records_from_map(raw.get("proposals"), "proposal_id")
        return {
            "proposal_count": raw.get("proposal_count", len(proposals)),
            "proposals": [_normalize_proposal(proposal) for proposal in proposals],
            "raw": raw,
        }
    if name == "datum.proposal.show":
        proposal = raw.get("proposal")
        if proposal is None:
            proposal = raw
        return {"proposal": _normalize_proposal(proposal), "raw": raw}
    if name == "datum.proposal.validate":
        return {
            "proposal_id": raw.get("proposal_id"),
            "can_apply": raw.get("can_apply"),
            "blocker_codes": raw.get("blocker_codes", []),
            "blockers": raw.get("blockers", []),
            "raw": raw,
        }
    return {
        "action": raw.get("action"),
        "proposal_id": raw.get("proposal_id"),
        "status": raw.get("status"),
        "review_status": raw.get("review_status"),
        "applied_operation_count": raw.get("applied_operation_count"),
        "transaction_id": raw.get("transaction_id"),
        "raw": raw,
    }


def _normalize_proposal(raw: Any) -> Any:
    if not isinstance(raw, dict):
        return raw
    return {
        "proposal_id": raw.get("proposal_id") or raw.get("id"),
        "project_id": raw.get("project_id"),
        "model_revision": raw.get("model_revision"),
        "status": raw.get("status"),
        "kind": raw.get("kind"),
        "source": raw.get("source"),
        "review": raw.get("review"),
        "finding_fingerprints": raw.get("finding_fingerprints", []),
        "operation_count": raw.get("operation_count", len(raw.get("operations", []))),
        "operations": raw.get("operations", []),
        "raw": raw,
    }


def _normalize_journal_result(name: str, raw: dict[str, Any]) -> dict[str, Any]:
    if name == "datum.journal.list":
        return {
            "transaction_count": raw.get("transaction_count", raw.get("count")),
            "cursor_index": raw.get("cursor_index"),
            "can_undo": raw.get("can_undo"),
            "can_redo": raw.get("can_redo"),
            "transactions": [
                _normalize_transaction(transaction)
                for transaction in raw.get("transactions", [])
            ],
            "raw": raw,
        }
    if name == "datum.journal.show":
        return {"transaction": _normalize_transaction(raw.get("transaction")), "raw": raw}
    return {
        "action": raw.get("action"),
        "status": raw.get("status"),
        "transaction_id": raw.get("transaction_id"),
        "cursor_index": raw.get("cursor_index"),
        "can_undo": raw.get("can_undo"),
        "can_redo": raw.get("can_redo"),
        "transaction": _normalize_transaction(raw.get("transaction")),
        "raw": raw,
    }


def _normalize_transaction(raw: Any) -> Any:
    if not isinstance(raw, dict):
        return raw
    operations = raw.get("operations", [])
    operation_count = (
        raw.get("operation_count", len(operations))
        if isinstance(operations, list)
        else raw.get("operation_count", operations)
    )
    return {
        "transaction_id": raw.get("transaction_id") or raw.get("id"),
        "project_id": raw.get("project_id"),
        "model_revision": raw.get("model_revision"),
        "status": raw.get("status"),
        "operation_count": operation_count,
        "operations": operations if isinstance(operations, list) else [],
        "raw": raw,
    }


def _normalize_component_instance_result(name: str, raw: dict[str, Any]) -> dict[str, Any]:
    return {
        "action": raw.get("action"),
        "component_instance": _normalize_component_instance(
            raw.get("component_instance")
            if isinstance(raw.get("component_instance"), dict)
            else {
                "component_instance_id": raw.get("component_instance"),
                "symbol": raw.get("symbol"),
                "symbol_id": raw.get("symbol_id"),
                "package": raw.get("package"),
                "package_id": raw.get("package_id"),
                "status": raw.get("status"),
                "binding": raw.get("binding"),
            }
        ),
        "raw": raw,
    }


def _normalize_query_result(name: str, raw: dict[str, Any]) -> dict[str, Any]:
    if name == "datum.query.component_instances":
        instances = _records_from_map(
            raw.get("component_instances"),
            "component_instance_id",
        )
        return {
            "component_instance_count": raw.get(
                "component_instance_count", len(instances)
            ),
            "component_instances": [
                _normalize_component_instance(instance) for instance in instances
            ],
            "raw": raw,
        }
    if name == "datum.query.relationships":
        relationships = raw.get("relationships")
        if not isinstance(relationships, list):
            relationships = _records_from_map(raw.get("statuses"), "relationship_id", "status")
        return {
            "relationship_count": raw.get("relationship_count", len(relationships)),
            "relationships": relationships,
            "raw": raw,
        }
    if name == "datum.query.variants":
        variants = _records_from_map(raw.get("populations"), "variant_id", "populations")
        return {
            "variant_count": raw.get("variant_count", len(variants)),
            "variants": variants,
            "raw": raw,
        }
    if name == "datum.query.import_map":
        entries = _records_from_map(raw.get("entries"), "import_id")
        return {
            "import_map_count": raw.get("import_map_count", len(entries)),
            "entries": entries,
            "raw": raw,
        }
    if name == "datum.query.panel_projections":
        return {
            "panel_projection_count": raw.get(
                "panel_projection_count", len(raw.get("panel_projections", []))
            ),
            "panel_projections": raw.get("panel_projections", []),
            "raw": raw,
        }
    if name == "datum.query.manufacturing_plans":
        return {
            "manufacturing_plan_count": raw.get(
                "manufacturing_plan_count", len(raw.get("manufacturing_plans", []))
            ),
            "manufacturing_plans": raw.get("manufacturing_plans", []),
            "raw": raw,
        }
    if name == "datum.query.output_jobs":
        return {
            "output_job_count": raw.get(
                "output_job_count", len(raw.get("output_jobs", []))
            ),
            "output_jobs": raw.get("output_jobs", []),
            "raw": raw,
        }
    return {**raw, "raw": raw}


def _normalize_component_instance(raw: Any) -> Any:
    if not isinstance(raw, dict):
        return raw
    return {
        "component_instance_id": raw.get("component_instance_id") or raw.get("id"),
        "project_id": raw.get("project_id"),
        "model_revision": raw.get("model_revision"),
        "symbol": raw.get("symbol"),
        "symbol_id": raw.get("symbol_id"),
        "package": raw.get("package"),
        "package_id": raw.get("package_id"),
        "status": raw.get("status"),
        "binding": raw.get("binding"),
        "raw": raw,
    }


def _records_from_map(raw: Any, id_key: str, value_key: str | None = None) -> list[Any]:
    if isinstance(raw, dict):
        records = []
        for key, value in raw.items():
            if isinstance(value, dict):
                records.append({id_key: key, **value})
            else:
                records.append({id_key: key, value_key or "value": value})
        return records
    if isinstance(raw, list):
        return raw
    return []
