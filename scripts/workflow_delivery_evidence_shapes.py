"""Closed PM041 proof/review shapes; no evidence execution or approval."""

from copy import deepcopy

from workflow_delivery_io import canonical_json, sha256
from workflow_delivery_shapes import (
    DIMENSIONS, array, blob, closed, commit_id, digest, enum, identifier,
    ids, ref, require, text, unique, version,
)


def proof_shape(value, path):
    closed(value, "schema_version contract_sha256 producer_session source_commit "
           "input_manifest build fixture environment results", path)
    version(value["schema_version"], path)
    digest(value["contract_sha256"], path)
    identifier(value["producer_session"], path)
    commit_id(value["source_commit"], path)
    for key in ("input_manifest", "fixture", "environment"):
        blob(value[key], path + "." + key)
    build = value["build"]
    closed(build, "command receipt toolchain source_clean", path + ".build")
    string_array(build["command"], path + ".build.command")
    blob(build["receipt"], path + ".build.receipt")
    text(build["toolchain"], path + ".build.toolchain")
    require(type(build["source_clean"]) is bool, path, "source_clean must be boolean")
    array(value["results"], path + ".results", code="WDQ-RESULT")
    for result in value["results"]:
        label = path + ".results"
        closed(result, "scenario_id outcome actual_visible actual_state assertions artifacts defects",
               label)
        identifier(result["scenario_id"], label)
        label += "." + result["scenario_id"]
        enum(result["outcome"], ("pass", "fail", "blocked", "unverified"), label)
        text(result["actual_visible"], label)
        text(result["actual_state"], label)
        ids(result["defects"], label, nonempty=False)
        array(result["artifacts"], label + ".artifacts", code="WDQ-ARTIFACT")
        for item in result["artifacts"]:
            blob(item, label)
        unique([b["path"] for b in result["artifacts"]], label)
        array(result["assertions"], label + ".assertions", code="WDQ-RESULT")
        for assertion in result["assertions"]:
            closed(assertion, "dimension expected observed outcome", label)
            enum(assertion["dimension"], DIMENSIONS, label)
            enum(assertion["outcome"], ("pass", "fail", "unverified"), label)
            text(assertion["expected"], label)
            text(assertion["observed"], label)
    unique([r["scenario_id"] for r in value["results"]], path + ".results", "WDQ-RESULT")
    return value


def string_array(value, path):
    array(value, path)
    for item in value:
        text(item, path)


def build_receipt_shape(value, path):
    closed(value, "binary_sha256 build_command toolchain input_manifest_sha256 exit_code", path)
    digest(value["binary_sha256"], path)
    digest(value["input_manifest_sha256"], path)
    text(value["toolchain"], path)
    string_array(value["build_command"], path)
    require(type(value["exit_code"]) is int and value["exit_code"] == 0,
            path, "successful build required", "WDQ-STALE")


def review_shape(value, path):
    closed(value, "schema_version packet_sha256 reviewer_session independent_of disposition "
           "replay findings owner_receipt", path)
    version(value["schema_version"], path)
    digest(value["packet_sha256"], path)
    identifier(value["reviewer_session"], path)
    ids(value["independent_of"], path)
    enum(value["disposition"], ("approve", "revise", "reject"), path)
    blob(value["replay"], path + ".replay")
    array(value["findings"], path + ".findings", nonempty=False)
    for finding in value["findings"]:
        closed(finding, "issue_id severity disposition_ref", path + ".findings")
        identifier(finding["issue_id"], path)
        enum(finding["severity"], ("blocking", "nonblocking"), path)
        if finding["disposition_ref"] is not None:
            ref(finding["disposition_ref"], path)
    unique([f["issue_id"] for f in value["findings"]], path)
    if value["owner_receipt"] is not None:
        ref(value["owner_receipt"], path)
    return value


def proof_sha256(proof):
    result = deepcopy(proof)
    result["results"].sort(key=lambda x: x["scenario_id"])
    for row in result["results"]:
        row["artifacts"].sort(key=lambda x: x["path"])
        row["defects"].sort()
        row["assertions"].sort(key=lambda x: (x["dimension"], x["expected"], x["observed"]))
    return sha256(canonical_json(result))


def review_sha256(review):
    result = deepcopy(review)
    del result["owner_receipt"]
    result["independent_of"].sort()
    result["findings"].sort(key=lambda x: x["issue_id"])
    return sha256(canonical_json(result))
