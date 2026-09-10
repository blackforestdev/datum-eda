"""Prepare/verify content-addressed runner bytes; never install local trust.

The prepared runner and hook are not an activation packet or an installation.
Existing and partial destinations are retained and refused, never overwritten.
"""

from workflow_delivery_bootstrap import bundle_manifest, verify_support
from workflow_delivery_io import canonical_json
from workflow_delivery_support_store import support_locations


def expected_bundle(root, authority):
    return bundle_manifest(root, authority)


def verify_bundle(root, authority):
    return verify_support(root, authority)[0]


def prepare_bundle(root, authority):
    locations = support_locations(root, authority)
    manifest, sources = expected_bundle(root, authority)
    # The full authority ID names a write-once destination. Never infer that an
    # existing partial directory is owned by this invocation or safe to repair.
    locations["trusted"].mkdir(parents=True, exist_ok=False)
    locations["scripts"].mkdir()
    locations["hooks"].mkdir()
    modes = {row["path"]: int(row["mode"], 8) for row in manifest["files"]}
    for name, raw in {**sources, "manifest.json": canonical_json(manifest)}.items():
        path = locations["trusted"] / name
        with path.open("xb") as stream:
            stream.write(raw)
        path.chmod(modes.get(name, 0o444))
    return verify_bundle(root, authority)
