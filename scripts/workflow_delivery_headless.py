"""Proposed PM042 infrastructure environment shape; no tool execution or approval."""

from workflow_delivery_io import DeliveryInputError
from workflow_delivery_shapes import array, closed, digest, enum, identifier, require, text, unique


def headless_shape(value):
    try:
        closed(value, "schema_version kind os toolchain transport terminal_size input_method "
               "tools reproduction_commands", "environment")
        require(type(value["schema_version"]) is int and value["schema_version"] == 2,
                "environment", "integer headless environment version 2 required")
        enum(value["kind"], ("headless",), "environment.kind")
        for field in ("os", "toolchain", "input_method"):
            text(value[field], "environment." + field)
        enum(value["transport"], ("pipes", "pty"), "environment.transport")
        size = value["terminal_size"]
        if value["transport"] == "pipes":
            require(size is None, "environment.terminal_size", "pipes have no terminal dimensions")
        else:
            require(type(size) is list and len(size) == 2
                    and all(type(n) is int and n > 0 for n in size),
                    "environment.terminal_size", "PTY requires observed positive columns/rows")
        array(value["tools"], "environment.tools")
        for tool in value["tools"]:
            closed(tool, "name path sha256 version", "environment.tools")
            identifier(tool["name"], "environment.tools.name")
            text(tool["path"], "environment.tools.path")
            digest(tool["sha256"], "environment.tools.sha256")
            text(tool["version"], "environment.tools.version")
        names = [tool["name"] for tool in value["tools"]]
        unique(names, "environment.tools")
        require({"interpreter", "git"} <= set(names), "environment.tools",
                "observed interpreter and git tool records required")
        array(value["reproduction_commands"], "environment.reproduction_commands")
        for command in value["reproduction_commands"]:
            text(command, "environment.reproduction_commands")
    except DeliveryInputError as error:
        raise DeliveryInputError("WDQ-ENVIRONMENT", error.path, error.detail) from error
    return value


def validate_headless(recorded, requested, *, category, toolchain, binary_sha256):
    require(category == "infrastructure", "environment",
            "headless environment cannot substitute for product native evidence", "WDQ-ENVIRONMENT")
    headless_shape(recorded)
    headless_shape(requested)
    require(recorded == requested, "environment",
            "requested environment differs; no equivalence is implicitly approved", "WDQ-ENVIRONMENT")
    require(recorded["toolchain"] == toolchain, "environment.toolchain",
            "environment/build toolchain mismatch", "WDQ-ENVIRONMENT")
    interpreter = next(tool for tool in recorded["tools"] if tool["name"] == "interpreter")
    require(interpreter["sha256"] == binary_sha256, "environment.tools.interpreter",
            "observed interpreter differs from build receipt binary", "WDQ-ENVIRONMENT")
