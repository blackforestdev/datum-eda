"""Hostile synthetic proof/review INPUT; never actual replay or acceptance."""

from workflow_delivery_authority import authority_sha256
from workflow_delivery_evidence_shapes import review_sha256
from workflow_delivery_io import canonical_json
from workflow_delivery_proof import packet_sha256
from workflow_delivery_tree import Tree

REVIEW_PATH = "docs/reviews/review.json"
EVENT_PATH = "evidence/original-events.json"
REPORT_ONLY_CASE = "INFRA-S01-03.report-only"
REPORT_CASES = {REPORT_ONLY_CASE, "INFRA-S01-03.enforce"}
REPORT_FIXTURE = "INFRA-S06-05.visible"
PROOF_INPUT_CASES = {
    "INFRA-S06-03.authority": (EVENT_PATH, "WDQ-STALE", "governing authority changed since observed run"),
    "INFRA-S06-03.source": ("evidence/inputs.json", "WDQ-INDEX", "changed relevant inputs"),
    "INFRA-S06-04.fixture": ("evidence/fixture.txt", "WDQ-STALE", "artifact hash mismatch"),
    "INFRA-S06-04.binary": ("evidence/registry.json", "WDQ-CONSUMER", "registry must identify tested binary"),
    "INFRA-S06-04.build": ("evidence/build.json", "WDQ-STALE", "build receipt disagrees with proof inputs/command/toolchain"),
    "INFRA-S06-04.registry": ("evidence/registry.json", "WDQ-CONSUMER", "registry must identify tested binary"),
    "INFRA-S06-05.inputs": (EVENT_PATH, "WDQ-RESULT", "input/method/session/binary correlation mismatch"),
    "INFRA-S06-05.dispatch": (EVENT_PATH, "WDQ-CONSUMER", "eligibility/enablement/dispatch registry disagreement"),
    "INFRA-S06-05.visible": (EVENT_PATH, "WDQ-RESULT", "event and observed result disagree"),
    "INFRA-S06-05.state": (EVENT_PATH, "WDQ-RESULT", "event and observed result disagree"),
    "INFRA-S06-05.artifact": ("evidence/original-roles.json", "WDQ-RESULT", "roles must bind every sibling artifact exactly"),
}
REVIEW_INPUT_CASES = {
    "INFRA-S04-10.authorized-nonblocking-deferral": (None, None, None),
    "INFRA-S04-02.implementation-session": (REVIEW_PATH, "WDQ-REVIEW", "reviewer not independent of implementation/original proof"),
    "INFRA-S04-02.producer-session": (REVIEW_PATH, "WDQ-REVIEW", "reviewer not independent of implementation/original proof"),
    "INFRA-S04-05.producer-defect": (REVIEW_PATH, "WDQ-DEFECT", "every proof defect requires review disposition"),
    "INFRA-S04-05.replay-defect": (REVIEW_PATH, "WDQ-DEFECT", "every proof defect requires review disposition"),
    "INFRA-S04-08.absent-receipt": (REVIEW_PATH, "WDQ-RECEIPT", "exact owner receipt required"),
    "INFRA-S04-08.wrong-receipt": ("docs/reviews/owner.md", "WDQ-RECEIPT", "exact owner ACCEPT response missing from referenced section"),
    "INFRA-S04-09.pending-disposition": (REVIEW_PATH, "WDQ-DEFECT", "undisposed defect dat-next"),
    "INFRA-S04-03.copied-events": (REVIEW_PATH, "WDQ-REVIEW", "copied event artifact is not independent replay"),
    "INFRA-S04-04.fixture": (REVIEW_PATH, "WDQ-REVIEW", "independent replay session/input identity mismatch"),
    "INFRA-S04-04.inputs": (REVIEW_PATH, "WDQ-REVIEW", "independent replay session/input identity mismatch"),
    "INFRA-S04-04.executable": (REVIEW_PATH, "WDQ-REVIEW", "independent replay must exercise the reviewed executable/build identity"),
}
REVIEW_INPUT_CASES.update(PROOF_INPUT_CASES)


def proof_expectation(variant, surface):
    path, code, detail = PROOF_INPUT_CASES[variant]
    if variant == "INFRA-S06-03.source" and surface not in ("C", "H"):
        code = "WDQ-STALE"
    return path, code, detail


def prepare_review_input(f, variant):
    if variant not in REVIEW_INPUT_CASES:
        raise ValueError("explicit implemented review-input variant required")
    tree = Tree(f.root)
    contract = f.contract
    proof = tree.json(contract["proof_path"])
    review = tree.json(contract["review_path"])
    if variant in PROOF_INPUT_CASES:
        prepare_proof_input(f, tree, proof, variant)
        return
    if variant.endswith(".authorized-nonblocking-deferral"):
        # Exact synthetic owner input before its fixture pin. Keep the defect
        # in both proof and findings; no actual owner response is manufactured.
        old_accept = f"ACCEPT TASK/A {review['packet_sha256']} {review_sha256(review)}"
        proof["results"][0]["defects"] = ["dat-next"]
        f.save(contract["proof_path"], proof)
        review["packet_sha256"] = packet_sha256(contract, proof, authority_sha256(tree, contract))
        receipt = review["owner_receipt"]["path"]
        review["findings"] = [{"issue_id": "dat-next", "severity": "nonblocking",
            "disposition_ref": {"path": receipt, "marker": "<!-- DEFERRAL -->"}}]
        new_accept = f"ACCEPT TASK/A {review['packet_sha256']} {review_sha256(review)}"
        raw = tree.read(receipt).decode()
        if raw.splitlines().count(old_accept) != 1:
            raise ValueError("exact original synthetic acceptance required")
        f.write(receipt, (raw.replace(old_accept, new_accept) +
            "\n<!-- DEFERRAL -->\n## Synthetic nonblocking disposition\nDEFER dat-next\n").encode())
    elif variant.endswith(".implementation-session"):
        review["reviewer_session"] = "writer"
    elif variant.endswith(".producer-session"):
        review["reviewer_session"] = proof["producer_session"]
    elif variant.endswith(".producer-defect"):
        proof["results"][0]["defects"] = ["dat-next"]
        f.save(contract["proof_path"], proof)
        review["packet_sha256"] = packet_sha256(contract, proof, authority_sha256(tree, contract))
    elif variant.endswith(".replay-defect"):
        replay = tree.json(review["replay"]["path"])
        replay["results"][0]["defects"] = ["dat-next"]
        review["replay"] = f.blob(review["replay"]["path"], canonical_json(replay))
    elif variant.endswith(".absent-receipt"):
        review["owner_receipt"] = None
    elif variant.endswith(".wrong-receipt"):
        receipt = review["owner_receipt"]["path"]
        exact = f"ACCEPT TASK/A {review['packet_sha256']} {review_sha256(review)}"
        raw = tree.read(receipt).decode()
        if raw.splitlines().count(exact) != 1:
            raise ValueError("exact original synthetic acceptance required")
        f.write(receipt, raw.replace(exact, "ACCEPT TASK/A " + "0" * 64 + " " + "0" * 64).encode())
    elif variant.endswith(".pending-disposition"):
        review["findings"] = [{"issue_id": "dat-next", "severity": "blocking",
                               "disposition_ref": None}]
    else:
        replay = tree.json(review["replay"]["path"])
        change_replay(f, tree, replay, variant)
        review["replay"] = f.blob(review["replay"]["path"], canonical_json(replay))
    f.save(contract["review_path"], review)


def prepare_proof_input(f, tree, proof, variant):
    """Alter synthetic input before its fixture pin, not real authority/proof."""
    from hashlib import sha256
    from workflow_delivery_native_test_support import replace_role
    if variant.endswith(".authority"):
        f.write("docs/authority.md", tree.read("docs/authority.md") + b"\nChanged synthetic requirement.\n")
        manifest = tree.json("specs/evidence_traceability_manifest.json")
        route = manifest["routes"][0]
        digest = sha256()
        for path in sorted(route["sources"] + route["consumers"]):
            digest.update(path.encode() + b"\0" + (f.root / path).read_bytes() + b"\0")
        route["reviewed_digest"] = digest.hexdigest()
        f.save("specs/evidence_traceability_manifest.json", manifest)
    elif variant.endswith(".source"):
        f.write("src/read.py", tree.read("src/read.py") + b"\n# Changed synthetic proof input.\n")
    elif variant.endswith(".fixture"):
        f.write(proof["fixture"]["path"], b"Changed synthetic fixture bytes.\n")
    elif variant.endswith((".binary", ".build")):
        receipt = tree.json(proof["build"]["receipt"]["path"])
        receipt["binary_sha256" if variant.endswith(".binary") else "input_manifest_sha256"] = "0" * 64
        proof["build"]["receipt"] = f.blob(proof["build"]["receipt"]["path"], canonical_json(receipt))
        f.save(f.contract["proof_path"], proof)
    else:
        roles = tree.json("evidence/original-roles.json")
        if variant.endswith(".registry"):
            registry = tree.json(roles["registry"]["path"])
            registry["binary_sha256"] = "0" * 64
            replace_role(f, proof, roles, "registry", registry)
        elif variant.endswith(".artifact"):
            roles["state"] = [f.blob("evidence/unlisted-state.txt", b"Unlisted synthetic state.\n")]
            index = f.blob("evidence/original-roles.json", canonical_json(roles))
            proof["results"][0]["artifacts"] = [index if row["path"] == index["path"] else row
                                                  for row in proof["results"][0]["artifacts"]]
            f.save(f.contract["proof_path"], proof)
        else:
            event = tree.json(EVENT_PATH)
            if variant.endswith(".inputs"):
                event["inputs"] = ["Unreviewed synthetic input"]
            elif variant.endswith(".dispatch"):
                event["dispatches"][0]["eligible"] = False
            else:
                event["actual_visible" if variant.endswith(".visible") else "actual_state"] = "Disagrees with result"
            replace_role(f, proof, roles, "events", event)


def change_replay(f, tree, replay, variant):
    if variant.endswith(".copied-events"):
        roles = tree.json("evidence/reviewer-roles.json")
        roles["events"] = tree.json("evidence/original-roles.json")["events"]
        index = f.blob("evidence/reviewer-roles.json", canonical_json(roles))
        replay["results"][0]["artifacts"] = (roles["events"] + roles["captures"]
            + roles["state"] + [roles["registry"], index])
    elif variant.endswith(".fixture"):
        replay["fixture"] = f.blob("evidence/replay-fixture.txt", b"different synthetic fixture\n")
    elif variant.endswith(".inputs"):
        # Keep a valid manifest list but change its exact serialized identity.
        # Bind a separate replay build receipt; never alter producer inputs.
        raw = tree.read(replay["input_manifest"]["path"])
        replay["input_manifest"] = f.blob("evidence/replay-inputs.json", raw + b"\n")
        receipt = tree.json(replay["build"]["receipt"]["path"])
        receipt["input_manifest_sha256"] = replay["input_manifest"]["sha256"]
        replay["build"]["receipt"] = f.blob("evidence/replay-build.json", canonical_json(receipt))
    elif variant.endswith(".executable"):
        receipt = tree.json(replay["build"]["receipt"]["path"])
        receipt["binary_sha256"] = "0" * 64
        replay["build"]["receipt"] = f.blob("evidence/replay-build.json", canonical_json(receipt))
    else:
        raise ValueError("explicit implemented replay mutation required")
