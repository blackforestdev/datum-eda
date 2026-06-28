#!/usr/bin/env python3
"""Lock the public MCP taxonomy to canonical datum.* families."""

from __future__ import annotations

import collections
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "mcp-server"))

from tools_catalog_data import (  # noqa: E402
    COMPATIBILITY_TOOL_SPECS,
    NON_JOURNALED_DAEMON_WRITE_METHODS,
    TOOL_BY_NAME,
    TOOLS,
)
from tools_catalog_retirement import (  # noqa: E402
    DEFAULT_HIDDEN_ALIAS_RETIREMENT_CRITERIA,
    HIDDEN_COMPATIBILITY_RETIREMENT_OVERRIDES,
)


EXPECTED_PUBLIC_PREFIX_COUNTS = {
    "datum.artifact": 11,
    "datum.check": 10,
    "datum.component_instance": 3,
    "datum.context": 4,
    "datum.journal": 4,
    "datum.library": 44,
    "datum.manufacturing": 6,
    "datum.output_job": 5,
    "datum.pcb": 44,
    "datum.pool": 3,
    "datum.proposal": 34,
    "datum.query": 48,
    "datum.replacement": 5,
    "datum.route": 21,
    "datum.schematic": 62,
    "datum.session": 4,
}

EXPECTED_PUBLIC_COUNT = sum(EXPECTED_PUBLIC_PREFIX_COUNTS.values())
EXPECTED_REGISTERED_COUNT = 479
EXPECTED_HIDDEN_COMPATIBILITY_COUNT = 171
ALLOWED_HIDDEN_RETIREMENT_STATUSES = {
    "retained_until_migration_plan",
    "deprecated",
    "scheduled_for_removal",
}
REQUIRED_DEPRECATED_HIDDEN_ALIASES = frozenset(HIDDEN_COMPATIBILITY_RETIREMENT_OVERRIDES)
GENERATED_FIXTURE_ROUTE_TOOLS = {
    "datum.route.write_strategy_fixture_suite",
    "datum.route.capture_strategy_baseline",
}
JOURNALED_ROUTE_WRITE_TOOLS = {
    "datum.route.apply": "journaled_route_apply",
    "datum.route.apply_selected": "journaled_route_apply",
    "datum.route.apply_proposal_artifact": "proposal_artifact_apply",
}
PROPOSAL_WRITE_TOOLS = {
    "datum.proposal.create": "proposal_metadata_write",
    "datum.proposal.create_draw_wire": "proposal_metadata_write",
    "datum.proposal.create_place_label": "proposal_metadata_write",
    "datum.proposal.create_place_symbol": "proposal_metadata_write",
    "datum.proposal.create_board_component_replacement": "proposal_metadata_write",
    "datum.proposal.create_board_component_replacements": "proposal_metadata_write",
    "datum.proposal.create_board_component_replacement_plan": "proposal_metadata_write",
    "datum.proposal.create_pool_library_object": "proposal_metadata_write",
    "datum.proposal.create_pool_unit": "proposal_metadata_write",
    "datum.proposal.create_pool_symbol": "proposal_metadata_write",
    "datum.proposal.create_pool_entity": "proposal_metadata_write",
    "datum.proposal.create_pool_padstack": "proposal_metadata_write",
    "datum.proposal.create_pool_package": "proposal_metadata_write",
    "datum.proposal.set_pool_package_pad": "proposal_metadata_write",
    "datum.proposal.set_pool_package_courtyard_rect": "proposal_metadata_write",
    "datum.proposal.set_pool_package_courtyard_polygon": "proposal_metadata_write",
    "datum.proposal.create_panel_projection": "proposal_metadata_write",
    "datum.proposal.update_panel_projection": "proposal_metadata_write",
    "datum.proposal.delete_panel_projection": "proposal_metadata_write",
    "datum.proposal.create_manufacturing_plan": "proposal_metadata_write",
    "datum.proposal.update_manufacturing_plan": "proposal_metadata_write",
    "datum.proposal.delete_manufacturing_plan": "proposal_metadata_write",
    "datum.proposal.create_output_job": "proposal_metadata_write",
    "datum.proposal.update_output_job": "proposal_metadata_write",
    "datum.proposal.delete_output_job": "proposal_metadata_write",
    "datum.proposal.review": "proposal_review_state_write",
    "datum.proposal.defer": "proposal_review_state_write",
    "datum.proposal.reject": "proposal_review_state_write",
    "datum.proposal.accept_apply": "proposal_gateway_apply",
    "datum.proposal.apply": "proposal_gateway_apply",
}
PROPOSAL_READ_TOOLS = {
    "datum.proposal.list",
    "datum.proposal.show",
    "datum.proposal.preview",
    "datum.proposal.validate",
}
PRODUCTION_PROPOSAL_WRITE_ALIASES = {
    "datum.manufacturing.create_panel_projection": "proposal_metadata_write",
    "datum.manufacturing.update_panel_projection": "proposal_metadata_write",
    "datum.manufacturing.delete_panel_projection": "proposal_metadata_write",
    "datum.manufacturing.create_plan": "proposal_metadata_write",
    "datum.manufacturing.update_plan": "proposal_metadata_write",
    "datum.manufacturing.delete_plan": "proposal_metadata_write",
    "datum.output_job.create_gerber_set": "proposal_metadata_write",
    "datum.output_job.create": "proposal_metadata_write",
    "datum.output_job.update": "proposal_metadata_write",
    "datum.output_job.delete": "proposal_metadata_write",
}


def main() -> int:
    public_names = [tool["name"] for tool in TOOLS]
    failures: list[str] = []

    if len(public_names) != EXPECTED_PUBLIC_COUNT:
        failures.append(
            f"public catalog count {len(public_names)} != expected {EXPECTED_PUBLIC_COUNT}"
        )
    if len(TOOL_BY_NAME) != EXPECTED_REGISTERED_COUNT:
        failures.append(
            f"registered tool count {len(TOOL_BY_NAME)} != expected {EXPECTED_REGISTERED_COUNT}"
        )
    if len(COMPATIBILITY_TOOL_SPECS) != EXPECTED_HIDDEN_COMPATIBILITY_COUNT:
        failures.append(
            "hidden compatibility count "
            f"{len(COMPATIBILITY_TOOL_SPECS)} != expected {EXPECTED_HIDDEN_COMPATIBILITY_COUNT}"
        )

    non_canonical = sorted(name for name in public_names if not name.startswith("datum."))
    if non_canonical:
        failures.append("public catalog contains non-datum names: " + ", ".join(non_canonical))

    prefix_counts = collections.Counter(".".join(name.split(".")[:2]) for name in public_names)
    if dict(sorted(prefix_counts.items())) != EXPECTED_PUBLIC_PREFIX_COUNTS:
        failures.append(
            "public prefix inventory changed:\n"
            f"  expected: {EXPECTED_PUBLIC_PREFIX_COUNTS}\n"
            f"  actual:   {dict(sorted(prefix_counts.items()))}"
        )

    public_dispatch_methods = {
        (TOOL_BY_NAME[name].get("x_dispatch_method") or name) for name in public_names
    }
    bypasses = sorted(public_dispatch_methods & NON_JOURNALED_DAEMON_WRITE_METHODS)
    if bypasses:
        failures.append(
            "public catalog dispatches to non-journaled write methods: " + ", ".join(bypasses)
        )

    for name in sorted(GENERATED_FIXTURE_ROUTE_TOOLS):
        spec = TOOL_BY_NAME.get(name)
        if not spec:
            failures.append(f"missing generated-fixture route tool {name}")
            continue
        if spec.get("authoring_boundary") != "generated_fixture_only":
            failures.append(f"{name} missing generated_fixture_only authoring boundary")
        if spec.get("write_path_policy") != (
            "direct project-shard writes are restricted to deterministic regression fixture generation"
        ):
            failures.append(f"{name} missing deterministic fixture write-path policy")

    for name, expected_class in sorted(JOURNALED_ROUTE_WRITE_TOOLS.items()):
        spec = TOOL_BY_NAME.get(name)
        if not spec:
            failures.append(f"missing journaled route write tool {name}")
            continue
        if spec.get("x_public_write_surface_class") != expected_class:
            failures.append(
                f"{name} missing x_public_write_surface_class={expected_class}"
            )
        if not spec.get("x_write_surface_evidence"):
            failures.append(f"{name} missing x_write_surface_evidence")
        description = str(spec.get("description") or "").lower()
        if "proposal" not in description or "journal" not in description:
            failures.append(f"{name} description must advertise the proposal journal gateway")
        if "directly" in description:
            failures.append(f"{name} description must not advertise direct mutation")

    public_proposal_names = {
        name for name in public_names if name.startswith("datum.proposal.")
    }
    classified_proposal_inventory = set(PROPOSAL_WRITE_TOOLS) | PROPOSAL_READ_TOOLS
    unknown_proposal_names = sorted(public_proposal_names - classified_proposal_inventory)
    if unknown_proposal_names:
        failures.append(
            "public datum.proposal aliases lack read/write inventory classification: "
            + ", ".join(unknown_proposal_names)
        )

    for name in sorted(PROPOSAL_READ_TOOLS):
        spec = TOOL_BY_NAME.get(name)
        if not spec:
            failures.append(f"missing proposal read tool {name}")
            continue
        if spec.get("x_public_write_surface_class"):
            failures.append(f"{name} read tool must not declare x_public_write_surface_class")

    for name, expected_class in sorted(PROPOSAL_WRITE_TOOLS.items()):
        spec = TOOL_BY_NAME.get(name)
        if not spec:
            failures.append(f"missing proposal write tool {name}")
            continue
        if spec.get("x_public_write_surface_class") != expected_class:
            failures.append(
                f"{name} missing x_public_write_surface_class={expected_class}"
            )
        if not spec.get("x_write_surface_evidence"):
            failures.append(f"{name} missing x_write_surface_evidence")
        description = str(spec.get("description") or "").lower()
        if "proposal" not in description:
            failures.append(f"{name} description must advertise proposal mediation")
        if "directly" in description:
            failures.append(f"{name} description must not advertise direct mutation")

    for name, expected_class in sorted(PRODUCTION_PROPOSAL_WRITE_ALIASES.items()):
        spec = TOOL_BY_NAME.get(name)
        if not spec:
            failures.append(f"missing proposal-mediated production write alias {name}")
            continue
        if spec.get("x_public_write_surface_class") != expected_class:
            failures.append(
                f"{name} missing x_public_write_surface_class={expected_class}"
            )
        if not spec.get("x_write_surface_evidence"):
            failures.append(f"{name} missing x_write_surface_evidence")
        method = spec.get("x_dispatch_method")
        if not str(method).endswith("_proposal"):
            failures.append(f"{name} must dispatch to a proposal builder, got {method}")
        description = str(spec.get("description") or "").lower()
        if "proposal" not in description:
            failures.append(f"{name} description must advertise proposal mediation")
        if "directly" in description:
            failures.append(f"{name} description must not advertise direct mutation")

    output_job_run_spec = TOOL_BY_NAME.get("datum.output_job.run")
    if not output_job_run_spec:
        failures.append("missing datum.output_job.run execution alias")
    elif output_job_run_spec.get("x_public_write_surface_class"):
        failures.append("datum.output_job.run must remain an execution surface, not a proposal write alias")

    hidden_names = {spec["name"] for spec in COMPATIBILITY_TOOL_SPECS}
    public_hidden_overlap = sorted(set(public_names) & hidden_names)
    if public_hidden_overlap:
        failures.append(
            "tools are both public and hidden compatibility aliases: "
            + ", ".join(public_hidden_overlap)
        )

    unclassified_hidden = sorted(
        spec["name"]
        for spec in COMPATIBILITY_TOOL_SPECS
        if spec.get("x_compatibility_visibility") != "hidden"
        or spec.get("x_retirement_status") not in ALLOWED_HIDDEN_RETIREMENT_STATUSES
        or not spec.get("x_retirement_criteria")
        or not spec.get("x_canonical_replacements")
    )
    if unclassified_hidden:
        failures.append(
            "hidden compatibility aliases lack retirement metadata: "
            + ", ".join(unclassified_hidden)
        )
    hidden_replacement_targets = []
    public_name_set = set(public_names)
    for spec in COMPATIBILITY_TOOL_SPECS:
        for replacement in spec.get("x_canonical_replacements", []):
            if replacement in public_name_set or str(replacement).startswith("pending:"):
                continue
            hidden_replacement_targets.append(f"{spec['name']} -> {replacement}")
    if hidden_replacement_targets:
        failures.append(
            "hidden compatibility aliases reference non-public replacement tools: "
            + ", ".join(sorted(hidden_replacement_targets))
        )

    compatibility_by_name = {str(spec["name"]): spec for spec in COMPATIBILITY_TOOL_SPECS}
    deprecated_hidden_aliases = {
        str(spec["name"])
        for spec in COMPATIBILITY_TOOL_SPECS
        if spec.get("x_retirement_status") == "deprecated"
    }
    if deprecated_hidden_aliases != REQUIRED_DEPRECATED_HIDDEN_ALIASES:
        failures.append(
            "deprecated hidden compatibility aliases must match explicit "
            "retirement overrides: "
            + ", ".join(
                sorted(deprecated_hidden_aliases ^ REQUIRED_DEPRECATED_HIDDEN_ALIASES)
            )
        )
    for name in sorted(REQUIRED_DEPRECATED_HIDDEN_ALIASES):
        spec = compatibility_by_name.get(name)
        if not spec:
            failures.append(f"missing deprecated hidden compatibility alias {name}")
            continue
        replacements = spec.get("x_canonical_replacements", [])
        replacement_label = ", ".join(str(replacement) for replacement in replacements)
        if spec.get("x_retirement_status") != "deprecated":
            failures.append(
                f"{name} must be deprecated now that {replacement_label} is public"
            )
        if not replacements:
            failures.append(f"{name} must name at least one canonical replacement")
        criteria = spec.get("x_retirement_criteria")
        if criteria == DEFAULT_HIDDEN_ALIAS_RETIREMENT_CRITERIA:
            failures.append(f"{name} must carry alias-specific retirement criteria")
        override = HIDDEN_COMPATIBILITY_RETIREMENT_OVERRIDES.get(name)
        if not override:
            failures.append(f"{name} must have an explicit retirement override")
        elif criteria != override["criteria"]:
            failures.append(f"{name} retirement criteria must match its override")

    if failures:
        print("MCP public taxonomy check failed:")
        for failure in failures:
            print(f"  - {failure}")
        return 1

    print(
        "MCP public taxonomy check passed "
        f"({EXPECTED_PUBLIC_COUNT} public, {EXPECTED_REGISTERED_COUNT} registered, "
        f"{EXPECTED_HIDDEN_COMPATIBILITY_COUNT} hidden compatibility)."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
