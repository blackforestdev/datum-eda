"""Exact terminal-renewal inputs for read-only I04 candidate preparation."""

import argparse
from pathlib import Path

SOURCE = "0abb325d9f06e7c8e4ea7b36e3a7ea2d8cd6f45c"
PRODUCER = "d88b2ee082effa4566e6d69b265f01df98d69390"
PACKET = "d7cef828f7b8a7bbf72ca488b666b14249816b317d4edd1761956ddc3779b5ce"
EVIDENCE = "docs/reviews/workflow-delivery-rollout/infrastructure/"
REVIEW_PREFIX = EVIDENCE + "terminal-renewal/independent/"


def reviewed_payloads(git, overlay, review_digest, inventory_digest):
    """Import immutable producer evidence and separately pinned review only."""
    from workflow_delivery_io import canonical_json, sha256
    overlay = Path(overlay)
    assert overlay.is_absolute() and overlay.is_dir() and not overlay.is_symlink()
    payloads = {}
    prefix = EVIDENCE + "terminal-renewal/"
    supplement = EVIDENCE + "terminal-owner/inventory.json"
    entries = git("ls-tree", "-rz", PRODUCER, "--", EVIDENCE + "proof.json", prefix, supplement)
    for entry in filter(None, entries.split(b"\0")):
        meta, raw_path = entry.split(b"\t", 1)
        mode, kind, oid = meta.split()
        path = raw_path.decode()
        assert mode == b"100644" and kind == b"blob"
        assert path in {EVIDENCE + "proof.json", supplement} or path.startswith(prefix)
        assert not path.startswith(REVIEW_PREFIX), "producer cannot supply independent review"
        payloads[path] = git("cat-file", "blob", oid.decode())
    assert {EVIDENCE + "proof.json", prefix + "raw.tar.xz", prefix + "inventory.json", supplement} <= payloads.keys()
    review_files = {}
    for path in sorted(overlay.rglob("*")):
        assert not path.is_symlink(), str(path)
        if path.is_dir():
            continue
        assert path.is_file(), str(path)
        relative = path.relative_to(overlay).as_posix()
        assert relative == EVIDENCE + "review.json" or relative.startswith(REVIEW_PREFIX), relative
        review_files[relative] = path.read_bytes()
    assert sha256(review_files[EVIDENCE + "review.json"]) == review_digest
    inventory = [{"path": path, "sha256": sha256(raw), "size": len(raw)}
                 for path, raw in sorted(review_files.items())]
    assert sha256(canonical_json(inventory)) == inventory_digest
    assert not payloads.keys() & review_files.keys()
    payloads.update(review_files)
    return payloads, inventory


if __name__ == "__main__":
    import sys
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    from prepare_sequencing_review_candidate import main
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--review-overlay", required=True)
    parser.add_argument("--review-digest", required=True)
    parser.add_argument("--review-inventory-digest", required=True)
    args = parser.parse_args()
    main(expected_step="WDQ-I04", profile="terminal", review_overlay=args.review_overlay,
         review_digest=args.review_digest, review_inventory_digest=args.review_inventory_digest)
