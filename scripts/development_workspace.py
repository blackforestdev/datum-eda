#!/usr/bin/env python3
"""Locate external development data; never treat a receipt as execution authority."""
from __future__ import annotations
import argparse
import json
import os
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


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
    relative = (path if path.is_absolute() else checkout / path).relative_to(checkout)
    if record is None:
        source = Path(os.environ.get('DATUM_FIXTURE_RELOCATION_MAP', checkout / '.datum/history-cleanup/target-relocation-map.json'))
        record = json.loads(source.read_text())
    if record.get('schema_version') != 1:
        raise ValueError('unsupported relocation map')
    for row in record['collections']:
        source = Path(row['source_relative'])
        if len(source.parts) != 2 or source.parts[0] != 'target' or '..' in source.parts:
            raise ValueError('invalid relocation source collection')
        if relative.is_relative_to(source):
            if row.get('disposition') == 'retired-released-test-output':
                raise ValueError('released temporary output was retired; regenerate through its current test')
            if row.get('state') != 'preserved':
                raise ValueError('fixture relocation is incomplete')
            return external(Path(row['destination']) / relative.relative_to(source), checkout)
    raise ValueError(f'no preserved relocation for {path}')


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
