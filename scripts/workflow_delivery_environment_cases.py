"""Malformed authority INPUT recipes; never owner promotion or proof repair."""

from copy import deepcopy

FIXTURE_VARIANTS = {
    "INFRA-S06-09.missing-key": "nonempty array required",
    "INFRA-S06-09.duplicate-key": "duplicate identity",
    "INFRA-S06-09.unknown-key": "must match every trusted enrolled key exactly",
    "INFRA-S06-12.selection-version": "integer environment selection version 2 required",
    "INFRA-S06-12.selection-kind": "environment selection kind required",
    "INFRA-S06-12.selection-fields": "closed fields required",
}
MIXED_VARIANT = "INFRA-S06-08.mixed"
SWAPPED_VARIANT = "INFRA-S06-10.swapped"
MIXED_VARIANTS = {MIXED_VARIANT, SWAPPED_VARIANT}
PREPARED_VARIANTS = set(FIXTURE_VARIANTS) | MIXED_VARIANTS


def malformed_selection(selection, variant):
    value = deepcopy(selection)
    if variant == "INFRA-S06-09.missing-key":
        value["environments"] = []
    elif variant == "INFRA-S06-09.duplicate-key":
        value["environments"].append(deepcopy(value["environments"][0]))
    elif variant == "INFRA-S06-09.unknown-key":
        value["environments"][0]["frontier_key"] = "UNKNOWN"
    elif variant == "INFRA-S06-12.selection-version":
        value["schema_version"] = True
    elif variant == "INFRA-S06-12.selection-kind":
        value["kind"] = "invalid-selection-kind"
    elif variant == "INFRA-S06-12.selection-fields":
        value["unexpected"] = "synthetic malformed authority input"
    else:
        raise ValueError("explicit implemented environment fixture variant required")
    return value


def case_uses_prepared_input(case_id, recipe):
    variant = recipe.get("fixture_variant")
    expected = case_id if case_id in PREPARED_VARIANTS else None
    if variant != expected:
        raise ValueError("case does not match retained fixture variant")
    return expected is not None
