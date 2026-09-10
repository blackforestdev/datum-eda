"""Versioned checkpoint order; legacy mappings remain byte-compatible."""

from workflow_delivery_shapes import closed, require


LEGACY_PHASES = ("ready", "activate", "verify", "accept")
REVIEW_PHASES = ("ready", "activate", "verify", "review", "accept")


def mapping_phases(delivery, key):
    if isinstance(delivery, dict) and "schema_version" in delivery:
        closed(delivery, "schema_version contract_path checkpoints", key, "WDQ-TRANSITION")
        require(type(delivery["schema_version"]) is int and delivery["schema_version"] == 2,
                key, "explicit delivery mapping requires integer schema_version 2",
                "WDQ-TRANSITION")
        return REVIEW_PHASES
    closed(delivery, "contract_path checkpoints", key, "WDQ-TRANSITION")
    return LEGACY_PHASES
