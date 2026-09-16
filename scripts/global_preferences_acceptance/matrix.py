"""Validate V2 instructions before they can define an acceptance proof."""

from itertools import product

from .inputs import array, canonical, closed, integer, require, text, unique, validation
from .inventory import BACKENDS, BUDGETS, GATES, PRODUCT_BOUNDARY, SCALES, VARIANTS, WINDOWS, subcases


@validation
def validate_matrix(matrix):
    closed(matrix, "schema frontier_task execution_step owner_acceptance_step "
           "requirement_markers status platform product_boundary durable_corpus "
           "candidate_resource_budgets acceptance_rule historical_execution_step contract protocol",
           "matrix")
    require(matrix["schema"] == "datum-global-preferences-production-acceptance-matrix-v2",
            "V2 matrix required for readiness")
    require(matrix["frontier_task"] == "GLOBAL-PREFERENCES-COMPLETION" and
            matrix["execution_step"] == "GP-CM05E" and
            matrix["owner_acceptance_step"] == "GP-CM05A" and
            matrix["historical_execution_step"] == "GP-CM05", "matrix task identity differs")
    require(canonical(matrix["product_boundary"]) == canonical(PRODUCT_BOUNDARY), "product boundary changed")
    require(matrix["requirement_markers"] == [
        f"<!-- REQ:GLOBAL-PREFERENCES-COMPLETION:{step} -->"
        for step in ("GP-CM05", "GP-CM05V")], "requirement markers differ")
    require(matrix["platform"] == {
        "os": "Linux", "architecture": "x86_64",
        "storage": "local durable filesystem with atomic rename and fsync support",
        "gui": "Wayland primary; X11 fallback", "network": "not required"}, "qualified platform differs")
    for field in ("status", "acceptance_rule", "contract"):
        text(matrix[field], field)
    require(matrix["contract"] == "specs/GLOBAL_PREFERENCES_PRODUCTION_ACCEPTANCE_CONTRACT.md",
            "matrix names a different acceptance contract")
    cases = array(matrix["durable_corpus"], "durable corpus")
    unique([r["id"] for r in cases], "case ids")
    require({r["id"] for r in cases} == set(VARIANTS), "case inventory differs")
    for row in cases:
        closed(row, "id source test authority test_anchor_role variants", "case")
        for field in ("source", "test", "authority", "test_anchor_role"):
            text(row[field], field)
        variants = array(row["variants"], "variants")
        unique([v["id"] for v in variants], "variant ids")
        require({v["id"] for v in variants} == set(VARIANTS[row["id"]]),
                f"{row['id']}: variant inventory differs")
        for variant in variants:
            closed(variant, "id surfaces assertions", "variant")
            surfaces, assertions = VARIANTS[row["id"]][variant["id"]]
            require(variant["surfaces"] == surfaces and variant["assertions"] == assertions,
                    f"{row['id']}/{variant['id']}: required surfaces/assertions changed")
    budgets = array(matrix["candidate_resource_budgets"], "budgets")
    unique([b["id"] for b in budgets], "budget ids")
    require({b["id"] for b in budgets} == set(BUDGETS), "budget inventory differs")
    for budget in budgets:
        closed(budget, "id metric limit protocol_section", "budget")
        text(budget["metric"], "budget metric")
        text(budget["protocol_section"], "protocol section")
        require(type(budget["limit"]) is dict, "budget limits must be an object")
        for value in budget["limit"].values():
            integer(value, "budget limit")
        require(budget["limit"] == BUDGETS[budget["id"]],
                f"{budget['id']}: numeric limits differ from approved baseline")
    protocol = matrix["protocol"]
    closed(protocol, "version contract evidence_schema warmup_operations "
           "samples_per_timed_variant_per_trial trials percentile compare_unrounded "
           "all_trials_must_pass allow_outlier_removal allow_successful_retry_substitution "
           "display_backends scale_factors window_sizes_logical_pixels native_scenarios "
           "gui_case_expansion lifecycle_cycles_per_trial reference_architecture "
           "reference_environment_rule ready_requires_owner_receipt "
           "accepted_requires_exact_owner_receipt required_gate_ids readiness_validator", "protocol")
    expected = {"version": 2, "warmup_operations": 10,
                "samples_per_timed_variant_per_trial": 100, "trials": 3,
                "lifecycle_cycles_per_trial": 100, "compare_unrounded": True,
                "all_trials_must_pass": True, "allow_outlier_removal": False,
                "allow_successful_retry_substitution": False,
                "ready_requires_owner_receipt": False, "accepted_requires_exact_owner_receipt": True,
                "display_backends": list(BACKENDS), "scale_factors": list(SCALES),
                "window_sizes_logical_pixels": [list(w) for w in WINDOWS],
                "native_scenarios": [f"N0{i}" for i in range(1, 7)],
                "required_gate_ids": GATES, "reference_architecture": "x86_64",
                "percentile": "nearest_rank_ceil_0.95_times_N_minus_1",
                "evidence_schema": "datum-global-preferences-production-evidence-v2",
                "contract": matrix["contract"]}
    for key, value in expected.items():
        require(type(protocol[key]) is type(value) and canonical(protocol[key]) == canonical(value),
                f"protocol {key}: required value differs")
    return matrix


def gui_coordinates():
    return tuple(product(BACKENDS, SCALES, WINDOWS))


def case_inventory():
    """Case identity includes the actual GUI backend, scale and logical dimensions."""
    result = {}
    for case, variants in VARIANTS.items():
        for variant, (surfaces, assertions) in variants.items():
            for surface in surfaces:
                for coordinate in gui_coordinates() if surface == "gui" else (None,):
                    for subcase in subcases(case, variant):
                        result[(case, variant, subcase, surface, coordinate)] = assertions
    return result
