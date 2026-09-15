"""Closed PM041 value shapes, shared by contracts and evidence records."""

import re

from workflow_delivery_io import DeliveryInputError, normalized_path


FOUNDATIONS = (
    "units_precision", "numeric_entry", "selection_identity", "grid_snap",
    "undo_cancel", "persistence_recovery", "library_connectivity", "settings_scope",
)
DIMENSIONS = (
    "normal", "invalid", "cancel", "scope", "precision", "undo_redo",
    "save_reopen", "accessibility", "failure_recovery",
)


def require(condition, path, detail, code="WDQ-CONTRACT"):
    if not condition:
        raise DeliveryInputError(code, path, detail)


def closed(value, fields, path, code="WDQ-CONTRACT"):
    require(type(value) is dict, path, "object required", code)
    expected = set(fields.split() if isinstance(fields, str) else fields)
    require(set(value) == expected, path,
            f"closed fields required; missing={sorted(expected - set(value))}, "
            f"unknown={sorted(set(value) - expected)}", code)
    return value


def text(value, path):
    require(type(value) is str and bool(value.strip()), path, "nonempty text required")
    return value


def identifier(value, path):
    text(value, path)
    require(re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]*", value) is not None,
            path, "ASCII ID required", "WDQ-IDENTITY")
    return value


def enum(value, choices, path):
    require(type(value) is str and value in choices, path, f"expected one of {choices}")
    return value


def version(value, path):
    require(type(value) is int and value == 1, path, "integer version 1 required")


def digest(value, path):
    require(type(value) is str and re.fullmatch(r"[0-9a-f]{64}", value) is not None,
            path, "lowercase SHA-256 required")
    return value


def commit_id(value, path):
    require(type(value) is str and re.fullmatch(r"[0-9a-f]{40}|[0-9a-f]{64}", value)
            is not None, path, "full Git object ID required")


def array(value, path, *, nonempty=True, code="WDQ-CONTRACT"):
    require(type(value) is list and (bool(value) or not nonempty), path,
            "nonempty array required" if nonempty else "array required", code)
    return value


def unique(values, path, code="WDQ-IDENTITY"):
    require(len(set(values)) == len(values), path, "duplicate identity", code)


def ids(value, path, *, nonempty=True):
    array(value, path, nonempty=nonempty)
    for index, item in enumerate(value):
        identifier(item, f"{path}[{index}]")
    unique(value, path)
    return value


def ref(value, path):
    closed(value, "path marker", path)
    normalized_path(value["path"])
    text(value["marker"], path + ".marker")


def refs(value, path):
    array(value, path)
    for index, item in enumerate(value):
        ref(item, f"{path}[{index}]")
    unique([(r["path"], r["marker"]) for r in value], path)


def blob(value, path):
    closed(value, "path sha256", path)
    normalized_path(value["path"])
    digest(value["sha256"], path + ".sha256")


def answer(value, path):
    closed(value, "disposition reason authority_refs", path, "WDQ-COVERAGE")
    enum(value["disposition"], ("required", "not_applicable"), path + ".disposition")
    text(value["reason"], path + ".reason")
    refs(value["authority_refs"], path + ".authority_refs")


def answers(value, keys, path):
    closed(value, keys, path, "WDQ-COVERAGE")
    for key in keys:
        answer(value[key], path + "." + key)


def all_refs(value):
    """Visit typed Ref-shaped values after the containing closed shape is checked."""
    if type(value) is dict:
        if set(value) == {"path", "marker"}:
            yield value
        else:
            for child in value.values():
                yield from all_refs(child)
    elif type(value) is list:
        for child in value:
            yield from all_refs(child)
