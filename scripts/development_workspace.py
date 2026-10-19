#!/usr/bin/env python3
"""Locate task-owned development data; never treat a receipt as execution authority."""
from __future__ import annotations
import argparse
import json
import os
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DATUM_COLLECTIONS = {"effective-process-delivery", "library-gui-delivery"}


def external(path: Path, checkout: Path = ROOT) -> Path:
    if not path.is_absolute() or '..' in path.parts:
        raise ValueError('workspace must be absolute without parent traversal')
    resolved = path.resolve(strict=False)
    if resolved.is_relative_to(checkout.resolve()):
        raise ValueError('workspace must remain outside the source checkout')
    return resolved


def root() -> Path:
    return external(Path(os.environ.get('DATUM_DEVELOPMENT_WORKSPACE', Path.home() / 'Documents/datum-eda-workspaces')))


def scratch(family: str) -> Path:
    if not family or not all(c.isascii() and (c.isalnum() or c in '-_') for c in family):
        raise ValueError('invalid workspace family')
    joined = root() / 'scratch' / family
    resolved = external(joined)
    if resolved != joined:
        raise ValueError('scratch path contains a symlink alias')
    return resolved


def relocated(path: str | Path, *, checkout: Path = ROOT, record: dict | None = None) -> Path:
    path = Path(path)
    if '..' in path.parts:
        raise ValueError('parent traversal in legacy fixture path')
    if record is None:
        replay = replay_input(path, checkout)
        if replay is not None:
            return replay
    relative = (path if path.is_absolute() else checkout / path).relative_to(checkout)
    if record is None:
        is_datum = relative.parts[:1] == ('.datum',)
        variable = 'DATUM_DEVELOPMENT_RELOCATION_MAP' if is_datum else 'DATUM_FIXTURE_RELOCATION_MAP'
        filename = 'development-relocation-map.json' if is_datum else 'target-relocation-map.json'
        source = Path(os.environ.get(variable, checkout / '.datum/history-cleanup' / filename))
        record = json.loads(source.read_text())
    if record.get('schema_version') != 1:
        raise ValueError('unsupported relocation map')
    for row in record['collections']:
        source = Path(row['source_relative'])
        if (len(source.parts) != 2 or '..' in source.parts
                or not (source.parts[0] == 'target'
                        or (source.parts[0] == '.datum' and source.parts[1] in DATUM_COLLECTIONS))):
            raise ValueError('invalid relocation source collection')
        if relative.is_relative_to(source):
            if row.get('disposition') == 'retired-released-test-output':
                raise ValueError('released temporary output was retired; regenerate through its current test')
            if row.get('state') != 'preserved':
                raise ValueError('fixture relocation is incomplete')
            return external(Path(row['destination']) / relative.relative_to(source), checkout)
    raise ValueError(f'no preserved relocation for {path}')


def replay_input(path: Path, checkout: Path = ROOT) -> Path | None:
    """Resolve approved frozen aliases only to an explicitly provisioned copy."""
    from provision_s5a_replay_inputs import INPUTS, catalogue, real_path, replay_location
    index = catalogue()
    if path.is_absolute():
        relative = None
        for base in [checkout, Path(index['legacy_checkout'])]:
            if path.is_relative_to(base):
                relative = path.relative_to(base)
                break
    else:
        relative = path
    if (relative is None or len(relative.parts) < 2 or relative.parts[0] != 'target'
            or relative.parts[1] not in index['aliases']):
        return None
    if 'DATUM_REPLAY_INPUTS' not in os.environ:
        raise ValueError('provision a fresh private replay root and set DATUM_REPLAY_INPUTS')
    root = replay_location(Path(os.environ['DATUM_REPLAY_INPUTS']), checkout)
    marker = real_path(root / 'index.json')
    if marker.read_bytes() != (INPUTS / 'index.json').read_bytes():
        raise ValueError('private replay catalogue differs from tracked inputs')
    result = real_path(root / Path(*relative.parts[1:]))
    if not result.exists():
        raise ValueError('replay path not provisioned')
    return result


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('operation', choices=['scratch', 'resolve'])
    parser.add_argument('path')
    args = parser.parse_args()
    try:
        print(scratch(args.path) if args.operation == 'scratch' else relocated(args.path))
    except (OSError, ValueError, KeyError) as error:
        parser.exit(1, f'Workspace path refused: {error}\n')


if __name__ == '__main__':
    main()
