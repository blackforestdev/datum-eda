"""Generated datum.* MCP tool specs loaded from the verb-registry catalog.

`datum_tool_catalog.json` is the checked-in projection of the single-source
verb registry (`crates/verb-registry`). It is emitted by
`cargo run -p datum-verb-registry --bin datum-verb-catalog -- --write` and
drift-gated by `-- --check` in `scripts/run_drift_gates.sh`.

Verb families migrate out of the hand-written catalogs one prefix at a time:
a prefix listed in ``MIGRATED_PREFIXES`` is owned by the generated catalog and
its hand-written entries must be deleted. The registry also declares verbs for
prefixes that are not yet migrated here (e.g. families whose GUI terminal
catalog is registry-projected while the MCP server still uses its hand-written
dicts); this loader filters strictly by ``MIGRATED_PREFIXES`` so those verbs
are never double-registered. Generated specs carry the exact same dict shape
as hand-written ones (``name``/``description``/``inputSchema`` plus
``x_dispatch_*`` and write-surface keys), so runtime dispatch through
``tool_dispatch.dispatch_tool_call`` and the ``server_runtime`` bridge methods
is byte-identical.
"""

from __future__ import annotations

import json
from pathlib import Path

# Verb-family prefixes whose tool specs come from the generated catalog.
MIGRATED_PREFIXES: frozenset[str] = frozenset({"datum.artifact"})

_CATALOG_PATH = Path(__file__).resolve().parent / "datum_tool_catalog.json"


def _tool_prefix(name: str) -> str:
    return ".".join(name.split(".")[:2])


def _spec_from_verb(verb: dict[str, object]) -> dict[str, object]:
    spec: dict[str, object] = {
        "name": verb["name"],
        "description": verb["description"],
        "inputSchema": verb["inputSchema"],
    }
    dispatch = verb.get("dispatch") or {}
    method = dispatch.get("method") if isinstance(dispatch, dict) else None
    if method:
        spec["x_dispatch_method"] = method
    if verb.get("dispatch_args"):
        spec["x_dispatch_args"] = list(verb["dispatch_args"])
    if verb.get("dispatch_defaults"):
        spec["x_dispatch_defaults"] = dict(verb["dispatch_defaults"])
    write_surface = verb.get("write_surface")
    if write_surface:
        spec["x_public_write_surface_class"] = write_surface["class"]
        spec["x_write_surface_evidence"] = write_surface["evidence"]
    return spec


def _load_generated_specs() -> list[dict[str, object]]:
    catalog = json.loads(_CATALOG_PATH.read_text(encoding="utf-8"))
    specs: list[dict[str, object]] = []
    seen: set[str] = set()
    for verb in catalog["verbs"]:
        name = str(verb["name"])
        if name in seen:
            raise RuntimeError(f"duplicate tool name in generated catalog: {name}")
        seen.add(name)
        if _tool_prefix(name) not in MIGRATED_PREFIXES:
            # Registry-declared but not yet MCP-migrated (e.g. GUI-terminal-only
            # families); the hand-written catalog still owns this prefix.
            continue
        if verb.get("status") != "public":
            raise RuntimeError(
                f"generated catalog verb {name} has unsupported status "
                f"{verb.get('status')!r}; hidden/retired generated verbs are not wired yet"
            )
        specs.append(_spec_from_verb(verb))
    return specs


GENERATED_TOOL_SPECS: list[dict[str, object]] = _load_generated_specs()
GENERATED_TOOL_NAMES: frozenset[str] = frozenset(
    str(spec["name"]) for spec in GENERATED_TOOL_SPECS
)


def generated_specs_for_prefix(prefix: str) -> list[dict[str, object]]:
    """Tool specs for one migrated verb-family prefix, sorted by name."""
    if prefix not in MIGRATED_PREFIXES:
        raise RuntimeError(f"prefix {prefix} is not migrated to the generated catalog")
    return [
        spec for spec in GENERATED_TOOL_SPECS if _tool_prefix(str(spec["name"])) == prefix
    ]


def reject_hand_written_duplicates(hand_written_names: list[str]) -> None:
    """Raise at import if a hand-written spec collides with a generated one."""
    duplicates = sorted(
        name for name in hand_written_names if name in GENERATED_TOOL_NAMES
    )
    if duplicates:
        raise RuntimeError(
            "tool names defined both hand-written and in the generated "
            "verb-registry catalog: " + ", ".join(duplicates)
        )
