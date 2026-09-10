"""Synthetic version-2 product readiness inputs, never native product proof."""

from copy import deepcopy

PRODUCT_READY = "INFRA-S03-04.product-valid"
PRODUCT_READINESS_CASES = {
    PRODUCT_READY: (None, None, None),
    "INFRA-S03-04.missing-handler": ("reader", "WDQ-CONSUMER", "required enabled consumer has no production handler"),
    "INFRA-S03-05.all-normal-na": ("next-contract", "WDQ-CONSUMER", "product delivery requires at least one normal enabled scenario"),
}


def product_readiness(item, contract, variant):
    if variant not in PRODUCT_READINESS_CASES:
        raise ValueError("explicit product readiness variant required")
    completion = item["completion"]
    ready, owner, activate = completion["steps"]
    verify, review, accept = deepcopy(activate), deepcopy(activate), deepcopy(owner)
    doc = item["governing_docs"][0]
    verify.update(id="NEXT-C04", depends_on=["NEXT-C03"],
                  requirement_refs=[{"path": doc, "marker": "NEXT-C04"}])
    review.update(id="NEXT-C05", depends_on=["NEXT-C04"],
                  requirement_refs=[{"path": doc, "marker": "NEXT-C05"}])
    accept.update(id="NEXT-C06", depends_on=["NEXT-C05"],
                  requirement_refs=[{"path": doc, "marker": "NEXT-C06"}], owner_input={
                      "response_format": "Fixture acceptance only", "requests": [{"id": "ACCEPT",
                          "question": "Accept fixture?", "recommended_response": "Fixture only",
                          "source_ref": {"path": doc, "marker": "ACCEPT"}}]})
    completion["steps"] = [ready, owner, activate, verify, review, accept]
    completion["delivery"] = {"schema_version": 2, "contract_path": "next.contract.json", "checkpoints": {
        "ready": "NEXT-C01", "activate": "NEXT-C03", "verify": "NEXT-C04", "review": "NEXT-C05", "accept": "NEXT-C06"}}
    contract["category"] = "product"
    for scenario in contract["scenarios"]:
        scenario["method"] = "native_input"
    if variant.endswith(".missing-handler"):
        contract["consumers"][0]["handler_ref"] = None
    elif variant.endswith(".all-normal-na"):
        for scenario in contract["scenarios"]:
            scenario["dimensions"]["normal"].update(disposition="not_applicable",
                reason="Synthetic attempted normal-behavior exemption, not accepted product scope")
    return ("\n<!-- REQ:NEXT:NEXT-C04 -->\n<!-- REQ:NEXT:NEXT-C05 -->\n"
            "<!-- REQ:NEXT:NEXT-C06 -->\n<!-- OWNER:NEXT:NEXT-C06:ACCEPT -->\n")
