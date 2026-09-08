"""Hostile synthetic review INPUT; never independent replay or owner acceptance."""

from workflow_delivery_authority import authority_sha256
from workflow_delivery_io import canonical_json
from workflow_delivery_proof import packet_sha256
from workflow_delivery_tree import Tree

REVIEW_PATH = "docs/reviews/review.json"
REVIEW_INPUT_CASES = {
    "INFRA-S04-02.implementation-session": (REVIEW_PATH, "WDQ-REVIEW", "reviewer not independent of implementation/original proof"),
    "INFRA-S04-02.producer-session": (REVIEW_PATH, "WDQ-REVIEW", "reviewer not independent of implementation/original proof"),
    "INFRA-S04-05.producer-defect": (REVIEW_PATH, "WDQ-DEFECT", "every proof defect requires review disposition"),
    "INFRA-S04-05.replay-defect": (REVIEW_PATH, "WDQ-DEFECT", "every proof defect requires review disposition"),
    "INFRA-S04-08.absent-receipt": (REVIEW_PATH, "WDQ-RECEIPT", "exact owner receipt required"),
    "INFRA-S04-09.pending-disposition": (REVIEW_PATH, "WDQ-DEFECT", "undisposed defect dat-next"),
    "INFRA-S04-03.copied-events": (REVIEW_PATH, "WDQ-REVIEW", "copied event artifact is not independent replay"),
    "INFRA-S04-04.fixture": (REVIEW_PATH, "WDQ-REVIEW", "independent replay session/input identity mismatch"),
    "INFRA-S04-04.inputs": (REVIEW_PATH, "WDQ-REVIEW", "independent replay session/input identity mismatch"),
    "INFRA-S04-04.executable": (REVIEW_PATH, "WDQ-REVIEW", "independent replay must exercise the reviewed executable/build identity"),
}


def prepare_review_input(f, variant):
    if variant not in REVIEW_INPUT_CASES:
        raise ValueError("explicit implemented review-input variant required")
    tree = Tree(f.root)
    contract = f.contract
    proof = tree.json(contract["proof_path"])
    review = tree.json(contract["review_path"])
    if variant.endswith(".implementation-session"):
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
    elif variant.endswith(".pending-disposition"):
        review["findings"] = [{"issue_id": "dat-next", "severity": "blocking",
                               "disposition_ref": None}]
    else:
        replay = tree.json(review["replay"]["path"])
        change_replay(f, tree, replay, variant)
        review["replay"] = f.blob(review["replay"]["path"], canonical_json(replay))
    f.save(contract["review_path"], review)


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
