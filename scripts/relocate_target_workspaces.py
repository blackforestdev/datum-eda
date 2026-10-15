#!/usr/bin/env python3
"""Preserve reviewed target collections outside Cargo, retiring only released tests."""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import stat
import subprocess

import development_workspace as workspace
import prune_development_output as prune
import run_cargo_guarded as guard

ROOT = guard.ROOT
# These are test-generated output collections, not pinned replay inputs. Their
# producers now use external scratch. Apply rechecks all tracked references.
RELEASED_TESTS = {
    'diagnostic-prepared-selection-tests', 'open-project-actual-worker-tests',
    'open-project-app-tests', 'open-project-protocol-tests',
    'proposal-refresh-tests', 'run-prepared-selection-tests',
}
UNCLASSIFIED = {'user-edit-placement', 'user-edit-qa-live'}
RESERVED = {'debug', 'release', 'doc', 'package', 'tmp', 'proofs', '.rustc_info.json', 'CACHEDIR.TAG'}
CONTROL = ROOT / '.datum/history-cleanup'
MAP = CONTROL / 'target-relocation-map.json'


def manifest(base: Path) -> list[dict]:
    records = []
    for path in [base, *sorted(base.rglob('*'))]:
        info = path.lstat()
        if not (stat.S_ISREG(info.st_mode) or stat.S_ISDIR(info.st_mode)):
            raise ValueError(f'nonregular collection entry: {path}')
        row = {'path': str(path.relative_to(base)), 'mode': info.st_mode,
               'kind': 'directory' if path.is_dir() else 'file'}
        if path.is_file():
            with path.open('rb') as stream:
                row.update(bytes=info.st_size, sha256=hashlib.file_digest(stream, 'sha256').hexdigest())
        records.append(row)
    return records


def canonical(value: object) -> bytes:
    return (json.dumps(value, sort_keys=True, separators=(',', ':')) + '\n').encode()


def write_record(path: Path, value: object) -> None:
    temporary = path.with_name(path.name + '.new')
    with temporary.open('xb') as output:
        output.write(canonical(value))
        output.flush()
        os.fsync(output.fileno())
    temporary.replace(path)


def references(name: str) -> list[str]:
    needle = ('target/' + name).encode()
    found = []
    files = subprocess.check_output(['git', 'ls-files', '-z'], cwd=ROOT).split(b'\0')
    for entry in files:
        if not entry:
            continue
        path = ROOT / os.fsdecode(entry)
        if path.is_file() and needle in path.read_bytes():
            found.append(os.fsdecode(entry))
    return found


def plan() -> dict:
    rows = []
    for path in sorted((ROOT / 'target').iterdir()):
        if path.name in RESERVED:
            continue
        if not path.is_dir() or path.is_symlink():
            raise ValueError(f'unclassified non-Cargo target entry: {path}')
        records = manifest(path)
        disposition = ('retired-released-test-output' if path.name in RELEASED_TESTS else
                       'user-owned-or-unclassified' if path.name in UNCLASSIFIED else 'retained-fixture')
        if disposition == 'user-owned-or-unclassified':
            destination = Path.home() / 'Documents/datum-projects/recovered-development-20261014' / path.name
        else:
            destination = workspace.root() / ('retired-tests-20261014' if path.name in RELEASED_TESTS else 'retained') / path.name
        refs = references(path.name)
        # The six producer names exist in this safety allowlist itself. No
        # other current tracked file may require their old generated outputs.
        if path.name in RELEASED_TESTS and any(p != 'scripts/relocate_target_workspaces.py' for p in refs):
            raise ValueError(f'released test output still referenced: {path.name}: {refs}')
        rows.append(dict(source_relative=str(path.relative_to(ROOT)), destination=str(workspace.external(destination)),
                         disposition=disposition, references=refs,
                         ownership=('unclassified; owner disposition required before deletion' if path.name in UNCLASSIFIED else 'generated development data; preserve retained task/input ownership'),
                         project_markers=sum(r['path'].endswith('project.json') and r['kind']=='file' for r in records),
                         file_bytes=sum(r.get('bytes', 0) for r in records),
                         manifest_sha256=hashlib.sha256(canonical(records)).hexdigest(), entries=records))
    return dict(schema_version=1, checkout=str(ROOT), maintenance_task='TARGET-WORKSPACE-SEPARATION', issue_id='dat-target-workspace-separation-1v8o', source_head=subprocess.check_output(['git','rev-parse','HEAD'], cwd=ROOT).decode().strip(), authority='owner-selected target cleanup; not execution or acceptance authority', collections=rows)


def apply(record: dict) -> None:
    if record.get('schema_version') != 1 or record.get('checkout') != str(ROOT):
        raise ValueError('plan belongs to a different checkout/schema')
    rows = record['collections']
    with guard.acquire_lock(guard.load_policy().lock_path, 0):
        prune.refuse_open_output([ROOT / 'target', workspace.root(), Path.home() / 'Documents/datum-projects/recovered-development-20261014'])
        if subprocess.check_output(['git', 'ls-files', '-z', '--', 'target/'], cwd=ROOT):
            raise ValueError('target contains tracked files')
        if MAP.exists() or MAP.is_symlink():
            raise ValueError('relocation map already exists; inspect recovery state instead of repeating')
        seen = set()
        for row in rows:
            rel = Path(row['source_relative']); source = ROOT / rel
            if len(rel.parts) != 2 or rel.parts[0] != 'target' or rel.name in RESERVED or rel.name in seen:
                raise ValueError('invalid/repeated source collection')
            seen.add(rel.name)
            if source.resolve() != source or manifest(source) != row['entries']:
                raise ValueError(f'source changed or unsafe: {source}')
            if hashlib.sha256(canonical(row['entries'])).hexdigest() != row['manifest_sha256']:
                raise ValueError('manifest digest mismatch')
            destination = Path(row['destination'])
            if workspace.external(destination) != destination or destination.exists() or destination.is_symlink():
                raise ValueError(f'destination occupied or unsafe: {destination}')
            expected = plan_destination(rel.name, row['disposition'])
            if destination != expected:
                raise ValueError('destination/disposition differs from reviewed storage policy')
            if row['disposition'] == 'retired-released-test-output':
                if rel.name not in RELEASED_TESTS or references(rel.name):
                    raise ValueError('released test now referenced or not approved')
        actual = {p.name for p in (ROOT / 'target').iterdir() if p.name not in RESERVED}
        if actual != seen:
            raise ValueError('target changed since plan')
        external_plan = workspace.root() / 'target-relocation-plan.json'
        external_plan.parent.mkdir(parents=True, exist_ok=True)
        with external_plan.open('xb') as output:
            output.write(canonical(record)); output.flush(); os.fsync(output.fileno())
        CONTROL.mkdir(parents=True, exist_ok=True)
        receipt = {k: v for k, v in record.items() if k != 'collections'}
        receipt.update(sealed_plan={'path': str(external_plan), 'sha256': hashlib.sha256(canonical(record)).hexdigest()}, method='atomic same-filesystem rename, manifest verification, then explicit released-test retirement', collections=[])
        write_record(MAP, receipt)
        for row in rows:
            source = ROOT / row['source_relative']; destination = Path(row['destination'])
            destination.parent.mkdir(parents=True, exist_ok=True)
            if destination.parent.resolve() != destination.parent or destination.parent.stat().st_dev != source.stat().st_dev:
                raise ValueError('destination must be a physical same-filesystem directory')
            # Persist intent and complete manifest before moving. An interrupted
            # operation has an explicit recovery state and is never auto-retried.
            manifest_path = CONTROL / (source.name + '.target-manifest.json')
            if manifest_path.exists():
                raise ValueError('unexpected preexisting collection manifest')
            write_record(manifest_path, row['entries'])
            entry = {k: v for k, v in row.items() if k != 'entries'}
            entry.update(state='prepared', manifest=str(manifest_path))
            receipt['collections'].append(entry); write_record(MAP, receipt)
            source.rename(destination)
            if manifest(destination) != row['entries']:
                raise ValueError(f'preserved manifest mismatch: {destination}')
            entry['state'] = 'preserved'; write_record(MAP, receipt)
            if row['disposition'] == 'retired-released-test-output':
                prune.refuse_open_output([destination])
                shutil.rmtree(destination)
                entry['state'] = 'retired'; write_record(MAP, receipt)
        write_record(workspace.root() / 'target-relocation-map.json', receipt)
        print(f"Completed {len(rows)} reviewed collections; map: {MAP}")


def plan_destination(name: str, disposition: str) -> Path:
    if disposition == 'user-owned-or-unclassified' and name in UNCLASSIFIED:
        return Path.home() / 'Documents/datum-projects/recovered-development-20261014' / name
    if disposition == 'retired-released-test-output' and name in RELEASED_TESTS:
        return workspace.root() / 'retired-tests-20261014' / name
    if disposition == 'retained-fixture' and name not in RELEASED_TESTS | UNCLASSIFIED:
        return workspace.root() / 'retained' / name
    raise ValueError('invalid collection disposition')


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--plan', type=Path, required=True)
    parser.add_argument('--apply', action='store_true')
    parser.add_argument('--released', action='store_true')
    args = parser.parse_args()
    try:
        if args.apply:
            if not args.released:
                parser.error('--apply requires --released confirmation for the six reviewed test collections')
            apply(json.loads(args.plan.read_text()))
        else:
            if args.plan.exists():
                raise ValueError('plan exists; inspect it rather than overwriting')
            record = plan(); write_record(args.plan, record)
            print(f"Preview: {len(record['collections'])} collections; full manifests/dispositions: {args.plan}")
        return 0
    except (OSError, ValueError, KeyError, guard.ResourcePolicyError) as error:
        print(f'Workspace relocation refused: {error}')
        return 75


if __name__ == '__main__':
    raise SystemExit(main())
