#!/usr/bin/env python3
"""Provision exact, private S5A replay inputs from tracked native seeds.

No external corpus, relocation map, archive, network, or compiled binary is used.
This prepares inputs only; it does not replay or qualify S5A.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath

ROOT = Path(__file__).resolve().parents[1]
INPUTS = ROOT / "crates/test-harness/testdata/selection/s5a_v1/replay-inputs"


def canonical(value):
    return (json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False)
            + "\n").encode()


def relative(value):
    path = PurePosixPath(value)
    if (not value or path.is_absolute() or any(p in (".", "..") for p in value.split("/"))
            or "\\" in value):
        raise ValueError("invalid input-relative path")
    return Path(*path.parts)


def real_path(path):
    """Refuse symlinks in every existing component, including dangling links."""
    if not path.is_absolute() or ".." in path.parts:
        raise ValueError("absolute path without traversal required")
    for part in (path, *path.parents):
        if part.is_symlink():
            raise ValueError("symlink input/output ancestor refused")
    return path


def dense_bytes(inputs):
    """Expand the four frozen dense files using their fixed 100000-text grid.

    The opaque historical metadata comes from the small tracked envelope. The
    population, operation order, inverse order, and compact serialization are
    explicit; every expanded file must match the original manifest digest.
    """
    template = json.loads((inputs / "dense-template.json").read_bytes())
    if template["schema_version"] != 1 or template["text_count"] != 100000:
        raise ValueError("unsupported dense recipe")
    sheet = template["sheet"]
    transactions = template["transactions"]
    tx = transactions[template["population_transaction"]]
    creates, deletes, ids = [], [], []
    for ordinal in range(100000):
        identity = f"50000000-0000-4000-8000-{0x1000 + ordinal:012x}"
        text = {"uuid": identity, "text": "I", "rotation": 0,
                "position": {"x": ordinal % 400 * 2000000,
                             "y": ordinal // 400 * 2000000}}
        sheet["texts"][identity] = text
        op = {"kind": "create_schematic_text", "sheet_id": sheet["uuid"],
              "text_id": identity, "text": text}
        creates.append(op)
        deletes.append({**op, "kind": "delete_schematic_text"})
        ids.append(identity)
    tx["operations"].extend(creates)
    tx["inverse_operations"] = list(reversed(deletes)) + tx["inverse_operations"]
    tx["diff"]["created"].extend(ids)
    transaction = canonical(tx)
    sheet_bytes = canonical(sheet)
    journal = b"".join(canonical(t) for t in transactions)
    return {".datum/journal/transactions.jsonl": journal,
            ".datum/revision/v1/blobs/sha256/" + hashlib.sha256(transaction).hexdigest(): transaction,
            ".datum/revision/v1/blobs/sha256/" + hashlib.sha256(sheet_bytes).hexdigest(): sheet_bytes,
            "schematic/selection.json": sheet_bytes}


def catalogue(inputs=INPUTS):
    real_path(inputs)
    real_path(inputs / "index.json")
    index = json.loads((inputs / "index.json").read_bytes())
    if index.get("schema_version") != 1:
        raise ValueError("unsupported replay input catalogue")
    seen = set()
    for row in index["files"]:
        path = relative(row["path"])
        if path.parts[0] not in index["aliases"] or row["path"] in seen:
            raise ValueError("unapproved or duplicate replay input")
        seen.add(row["path"])
        if "source" in row:
            real_path(inputs / relative(row["source"]))
        elif "dense" not in row:
            raise ValueError("missing input source")
    return index


def checked_bytes(row, inputs, dense):
    data = ((inputs / relative(row["source"])).read_bytes() if "source" in row
            else dense[row["dense"]])
    if len(data) != row["bytes"] or hashlib.sha256(data).hexdigest() != row["sha256"]:
        raise ValueError("replay input differs from frozen hash: " + row["path"])
    return data


def verify_history(index, inputs, dense):
    """Verify every head's full generation ancestry and content-addressed blobs.

    A source-invariance test's allowlist can be smaller than its backing history;
    the catalogue must contain both. This never edits or repairs native history.
    """
    rows = {r['path']: r for r in index['files']}

    def load(path):
        if path not in rows:
            raise ValueError('incomplete revision closure: ' + path)
        return checked_bytes(rows[path], inputs, dense)

    generations = 0
    for path in rows:
        if not path.endswith('/.datum/revision/v1/head.json'):
            continue
        prefix = path.removesuffix('head.json')
        head = json.loads(load(path))
        transactions = {}
        records = {}
        for generation_path in rows:
            if not generation_path.startswith(prefix + 'generations/'):
                continue
            generation = json.loads(load(generation_path))
            root = generation['integrity_root']
            if (not root.startswith('sha256:') or generation['project_id'] != head['project_id']
                    or generation_path != prefix + 'generations/' + root.removeprefix('sha256:') + '.json'):
                raise ValueError('revision identity/ancestry mismatch')
            material = {k: v for k, v in generation.items() if k != 'integrity_root'}
            for key in ['parent_transaction_id', 'parent_integrity_root']:
                material.setdefault(key, None)
            if 'sha256:' + hashlib.sha256(canonical(material)).hexdigest() != root:
                raise ValueError('revision generation material digest mismatch')
            blobs = [generation['transaction_blob'], generation.get('authority_snapshot_blob')]
            blobs += [a.get('postimage_blob') for a in generation['affected_shards']]
            for digest in filter(None, blobs):
                blob = load(prefix + 'blobs/sha256/' + digest.removeprefix('sha256:'))
                if 'sha256:' + hashlib.sha256(blob).hexdigest() != digest:
                    raise ValueError('revision content-addressed blob mismatch')
            tx = json.loads(load(prefix + 'blobs/sha256/' + generation['transaction_blob'].removeprefix('sha256:')))
            for field in ['transaction_id', 'before_model_revision', 'after_model_revision']:
                if tx[field] != generation[field]:
                    raise ValueError('revision transaction mismatch')
            transactions[tx['transaction_id']] = tx
            records[root] = generation
            generations += 1
        # Check every stored parent edge, including backing generations for
        # journal prefixes/redo tails that are not on the current head's chain.
        for record in records.values():
            parent = record.get('parent_integrity_root')
            if parent:
                load(prefix + 'generations/' + parent.removeprefix('sha256:') + '.json')
                if records[parent]['transaction_id'] != record['parent_transaction_id']:
                    raise ValueError('revision parent transaction mismatch')
        root, transaction_id = head['integrity_root'], head['transaction_id']
        seen = set()
        while root:
            if root in seen:
                raise ValueError('invalid revision ancestry')
            seen.add(root)
            load(prefix + 'generations/' + root.removeprefix('sha256:') + '.json')
            generation = records[root]
            if generation['transaction_id'] != transaction_id:
                raise ValueError('revision identity/ancestry mismatch')
            root, transaction_id = generation.get('parent_integrity_root'), generation.get('parent_transaction_id')
        project = path.removesuffix('.datum/revision/v1/head.json')
        journal_lines = load(project + '.datum/journal/transactions.jsonl').splitlines(keepends=True)
        journal = [json.loads(line) for line in journal_lines]
        cursor = json.loads(load(project + '.datum/journal/cursor.json'))['applied_transaction_count']
        if not 0 <= cursor <= len(journal):
            raise ValueError('invalid journal cursor')
        inherited = index.get('inherited_journal_prefixes', {}).get(project.rstrip('/'))
        prefix_count = 0
        if inherited:
            prefix_count = len(inherited['transaction_ids'])
            prefix_bytes = b''.join(journal_lines[:prefix_count])
            boundary = records.get(inherited['first_stored_generation'])
            pinned = all(hashlib.sha256(load(project + role)).hexdigest() == inherited[key]
                         for role, key in [('.datum/journal/transactions.jsonl', 'journal_sha256'),
                                           ('.datum/journal/cursor.json', 'cursor_sha256'),
                                           ('.datum/revision/v1/head.json', 'head_sha256')])
            store_paths = sorted(p.removeprefix(project) for p in rows if p.startswith(prefix))
            missing = [t['transaction_id'] for t in journal if t['transaction_id'] not in transactions]
            if (prefix_count != 5 or len(prefix_bytes) != inherited['bytes']
                    or not pinned or store_paths != inherited['store_paths']
                    or missing != inherited['transaction_ids']
                    or hashlib.sha256(prefix_bytes).hexdigest() != inherited['sha256']
                    or [t['transaction_id'] for t in journal[:prefix_count]] != inherited['transaction_ids']
                    or sorted(records) != inherited['stored_generation_roots']
                    or set(inherited['transaction_ids']) & transactions.keys()
                    or not boundary or boundary.get('parent_integrity_root') is not None
                    or boundary.get('parent_transaction_id') != journal[prefix_count - 1]['transaction_id']
                    or boundary['before_model_revision'] != journal[prefix_count - 1]['after_model_revision']
                    or boundary['transaction_id'] != journal[prefix_count]['transaction_id']):
                raise ValueError('frozen inherited journal prefix/boundary mismatch')
        # Include the complete journal, not only its applied prefix. Any future
        # redo-tail fixture needs its backing generations too; never drop it.
        # Six explicitly pinned historical clones inherited five journal entries
        # without copying the store. Their exact prefix and recovery boundary are
        # checked above; do not fabricate generations or add recovery candidates.
        for transaction in journal[prefix_count:]:
            if transactions.get(transaction['transaction_id']) != transaction:
                raise ValueError('journal transaction lacks exact backing history: '
                                 + project + transaction['transaction_id'])
        if cursor != len(journal) or head['transaction_id'] != journal[-1]['transaction_id']:
            raise ValueError('frozen cursor/head does not bind complete journal tip')
    return generations


def verify(inputs=INPUTS):
    index = catalogue(inputs)
    real_path(inputs / "dense-template.json")
    dense = dense_bytes(inputs)
    for row in index["files"]:
        checked_bytes(row, inputs, dense)
    verify_history(index, inputs, dense)
    return index, dense


def replay_location(path, checkout=ROOT):
    """Admit external copies or one labelled frozen-copy root in task scratch.

    This exception belongs only to catalogue-backed S5A replay copies. General
    development workspaces and newly generated fixtures retain their own rules.
    """
    path = real_path(path).resolve()
    checkout = real_path(checkout).resolve()
    cargo_target = Path(os.environ.get("CARGO_TARGET_DIR", checkout / "target")).absolute().resolve()
    if path.is_relative_to(cargo_target) or cargo_target.is_relative_to(path):
        raise ValueError("replay inputs must remain separate from Cargo output")
    if path.is_relative_to(checkout):
        parts = path.relative_to(checkout).parts
        if (len(parts) != 4 or parts[:2] != (".datum", "scratch")
                or parts[3] != "s5a-replay-inputs"
                or not all(c.isascii() and (c.isalnum() or c in "-_") for c in parts[2])):
            raise ValueError("checkout replay copies require labelled .datum/scratch/<owner>/s5a-replay-inputs")
    return path


def provision(output, inputs=INPUTS):
    output = replay_location(output, ROOT)
    if output.exists():
        raise ValueError("fresh replay output required")
    # Validate all seeds and generated bytes before creating any output.
    index, dense = verify(inputs)
    output.parent.mkdir(parents=True, exist_ok=True)
    real_path(output)
    output.mkdir()  # create_new behavior; no merging or overwriting
    for row in index["files"]:
        path = output / relative(row["path"])
        path.parent.mkdir(parents=True, exist_ok=True)
        with path.open("xb") as stream:
            stream.write(checked_bytes(row, inputs, dense))
    actual = {p.relative_to(output).as_posix() for p in output.rglob('*') if p.is_file()}
    if actual != {r['path'] for r in index['files']}:
        raise ValueError('provisioned native inventory differs from frozen catalogue')
    # Copy the exact catalogue as the completion marker, written last. Readers
    # compare it with their checkout's tracked catalogue before resolving paths.
    with (output / "index.json").open("xb") as stream:
        stream.write((inputs / "index.json").read_bytes())
    return index


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path,
                        help="fresh absolute external root or .datum/scratch/<owner>/s5a-replay-inputs")
    args = parser.parse_args()
    try:
        index = provision(args.output) if args.output else verify()[0]
    except (OSError, ValueError, KeyError) as error:
        parser.exit(1, f"replay inputs refused: {error}\n")
    print(f"Verified {len(index['files'])} exact files across {len(index['aliases'])} legacy collections.")
    if args.output:
        print(f"DATUM_REPLAY_INPUTS={args.output}")


if __name__ == "__main__":
    main()
