"""Strict evidence parsing and digest-bound, nonredirected artifact reads."""

from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import stat
from functools import wraps

from workflow_delivery_io import parse_json as parse_shared_json


class EvidenceError(ValueError):
    """An input failed a concrete acceptance requirement."""


def validation(function):
    """Malformed nested inputs and inaccessible artifacts fail at the API boundary."""
    @wraps(function)
    def checked(*args, **kwargs):
        try:
            return function(*args, **kwargs)
        except (KeyError, TypeError, IndexError, OSError, OverflowError) as error:
            raise EvidenceError(f"{function.__name__}: malformed or inaccessible input: {error}") from error
    return checked


def require(condition, message):
    if not condition:
        raise EvidenceError(message)


def closed(value, fields, label):
    require(type(value) is dict and set(value) == set(fields.split()),
            f"{label}: expected exactly {fields}")
    return value


def text(value, label):
    require(type(value) is str and bool(value.strip()), f"{label}: nonempty text required")
    return value


def integer(value, label, minimum=0):
    require(type(value) is int and value >= minimum, f"{label}: integer >= {minimum} required")
    return value


def array(value, label, *, nonempty=True):
    require(type(value) is list and (value or not nonempty), f"{label}: list required")
    return value


def unique(values, label):
    require(len(values) == len(set(values)), f"{label}: duplicate identity")


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def sha(value, label):
    require(type(value) is str and re.fullmatch(r"[0-9a-f]{64}", value),
            f"{label}: lowercase SHA-256 required")
    return value


def revision(value):
    require(type(value) is str and re.fullmatch(r"[0-9a-f]{40}", value),
            "candidate: full commit identity required")
    return value


def parse(raw, label):
    try:
        return parse_shared_json(raw, label)
    except ValueError as error:
        raise EvidenceError(f"{label}: invalid JSON: {error}") from error


def canonical(value):
    """V2 report identity has no trailing newline; raw artifacts retain theirs."""
    try:
        return json.dumps(value, sort_keys=True, separators=(",", ":"),
                          ensure_ascii=False, allow_nan=False).encode("utf-8")
    except (ValueError, TypeError, UnicodeError) as error:
        raise EvidenceError(f"invalid canonical JSON: {error}") from error


def relative(value):
    text(value, "artifact path")
    path = PurePosixPath(value)
    require(bool(path.parts) and not path.is_absolute() and str(path) == value and
            all(part not in (".", "..") for part in path.parts) and
            "\\" not in value and "\x00" not in value,
            f"unsafe artifact path: {value!r}")
    return path


def read_file(root, name):
    """Open every component without following links, including race replacements."""
    path = relative(name)
    root = Path(root).absolute()
    require(root == root.resolve(), "artifact root is redirected")
    descriptor = os.open(root, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        for part in path.parts[:-1]:
            child = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW,
                            dir_fd=descriptor)
            os.close(descriptor)
            descriptor = child
        child = os.open(path.parts[-1], os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK,
                        dir_fd=descriptor)
        with os.fdopen(child, "rb") as stream:
            require(stat.S_ISREG(os.fstat(stream.fileno()).st_mode),
                    f"artifact is not a regular file: {name}")
            return stream.read()
    finally:
        os.close(descriptor)


class Bundle:
    """All reads must belong to the declared exact artifact inventory."""

    def __init__(self, root, inventory):
        self.root = Path(root)
        self.entries = {}
        self.references_by_digest = {}
        self.validated_sizes = {}
        for entry in array(inventory, "artifacts"):
            closed(entry, "path sha256", "artifact")
            name = str(relative(entry["path"]))
            require(name not in self.entries, f"duplicate artifact: {name}")
            self.entries[name] = sha(entry["sha256"], name)
            self.references_by_digest[entry["sha256"]] = entry
        # Validate unused entries too; an invalid attachment cannot hide in a bundle.
        for name, expected in self.entries.items():
            raw = self.read({"path": name, "sha256": expected})
            self.validated_sizes[expected] = len(raw)

    def read(self, reference):
        closed(reference, "path sha256", "artifact reference")
        name = str(relative(reference["path"]))
        require(self.entries.get(name) == sha(reference["sha256"], name),
                f"artifact reference is missing or differs: {name}")
        raw = read_file(self.root, name)
        require(digest(raw) == reference["sha256"], f"artifact digest differs: {name}")
        return raw

    def json(self, reference):
        return parse(self.read(reference), reference["path"])
