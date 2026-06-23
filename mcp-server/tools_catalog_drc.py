"""Legacy DRC MCP compatibility tool schema fragments."""

DRC_RULE_NAMES = [
    "Connectivity",
    "ClearanceCopper",
    "TrackWidth",
    "ViaHole",
    "ViaAnnularRing",
    "SilkClearance",
    "ProcessAperture",
]

RUN_DRC_INPUT_SCHEMA = {
    "type": "object",
    "properties": {
        "rules": {"type": "array", "items": {"type": "string", "enum": DRC_RULE_NAMES}}
    },
}
