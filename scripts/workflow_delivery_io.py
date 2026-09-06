"""Read-only JSON, path and digest primitives for PM041 delivery validation.

This module does not establish enrollment, readiness, trust or acceptance.
Worktree reads are explicit; a future staged/candidate reader must supply its own
tree bytes rather than silently falling back to these filesystem helpers.
"""

from __future__ import annotations

import hashlib
import json
import math
from pathlib import Path, PurePosixPath


class DeliveryInputError(ValueError):
    """A stable diagnostic with an owning input path, never an automatic repair."""

    def __init__(self, code: str, path: str, detail: str):
        self.code = code
        self.path = path
        self.detail = detail
        super().__init__(f"{code}: {path}: {detail}")


def _object(pairs: list[tuple[str, object]]) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate object key {key!r}")
        result[key] = value
    return result


def _float(token: str) -> float:
    result = float(token)
    if not math.isfinite(result):
        raise ValueError("nonfinite JSON number")
    return result


def _constant(token: str) -> object:
    raise ValueError(f"nonstandard JSON constant {token}")


def parse_json(raw: bytes, path: str) -> object:
    """Reject invalid UTF-8, duplicate keys and all nonfinite representations."""
    try:
        value = json.loads(
            raw.decode("utf-8"), object_pairs_hook=_object,
            parse_float=_float, parse_constant=_constant,
        )
        # JSON escape sequences can encode lone surrogates despite valid input
        # UTF-8. Such values cannot have the required canonical UTF-8 identity.
        _json_value(value)
        return value
    except (ValueError, UnicodeError, RecursionError) as error:
        raise DeliveryInputError("WDQ-CONTRACT", path, str(error)) from error


def _json_value(value: object) -> None:
    """Prevent Python-only values from silently acquiring JSON identities."""
    if value is None or type(value) in (bool, int):
        return
    if type(value) is str:
        value.encode("utf-8")
    elif type(value) is float:
        if not math.isfinite(value):
            raise ValueError("nonfinite JSON number")
    elif type(value) is list:
        for child in value:
            _json_value(child)
    elif type(value) is dict:
        for key, child in value.items():
            if type(key) is not str:
                raise ValueError("JSON object keys must be strings")
            key.encode("utf-8")
            _json_value(child)
    else:
        raise ValueError(f"not a JSON value: {type(value).__name__}")


def canonical_json(value: object) -> bytes:
    """PM041 canonical JSON; array ordering remains the shape owner's job."""
    try:
        _json_value(value)
        return (json.dumps(
            value, sort_keys=True, separators=(",", ":"), ensure_ascii=False,
            allow_nan=False,
        ) + "\n").encode("utf-8")
    except (ValueError, UnicodeError, RecursionError) as error:
        raise DeliveryInputError("WDQ-CONTRACT", "<canonical-json>", str(error)) from error


def sha256(raw: bytes) -> str:
    """Raw-byte Blob identity; callers explicitly canonicalize JSON when required."""
    return hashlib.sha256(raw).hexdigest()


def normalized_path(path: str) -> PurePosixPath:
    """Validate a reference without interpreting it as a command or a URL."""
    if not isinstance(path, str) or not path or "\0" in path:
        raise DeliveryInputError("WDQ-ARTIFACT", str(path), "nonempty POSIX path required")
    parsed = PurePosixPath(path)
    if (parsed.is_absolute() or str(parsed) != path
            or any(part in (".", "..", "") for part in path.split("/"))
            or "\\" in path or ":" in path):
        raise DeliveryInputError("WDQ-ARTIFACT", path, "normalized repository path required")
    return parsed


def worktree_path(root: Path, path: str, *, must_exist: bool = True) -> Path:
    """Resolve symlinks before containment checks, including future artifact paths."""
    normalized_path(path)
    try:
        base = root.resolve(strict=True)
        resolved = (base / path).resolve(strict=must_exist)
        if not base.is_dir() or not resolved.is_relative_to(base):
            raise ValueError("reference escapes repository root")
        return resolved
    except (OSError, ValueError, RuntimeError) as error:
        raise DeliveryInputError("WDQ-ARTIFACT", path, str(error)) from error


def read_worktree_bytes(root: Path, path: str) -> bytes:
    """Read one existing regular file; never fetch, generate, or execute a reference."""
    resolved = worktree_path(root, path)
    try:
        if not resolved.is_file():
            raise ValueError("regular file required")
        return resolved.read_bytes()
    except (OSError, ValueError) as error:
        raise DeliveryInputError("WDQ-ARTIFACT", path, str(error)) from error
