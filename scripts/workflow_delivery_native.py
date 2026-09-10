"""Typed artifact protocol for PM041 correlation, never a visual-quality oracle.

The versioned artifact index is itself a hashed result Blob. It binds roles to
exact sibling Blobs, so filenames and MIME guesses never establish independence.
Production runners emit these records; hermetic fixtures test refusal mechanics
but cannot establish the production origin of an export or a user's experience.
"""

from workflow_delivery_authority import authority_sha256
from workflow_delivery_headless import validate_headless
from workflow_delivery_io import parse_json
from workflow_delivery_proof import read_blob
from workflow_delivery_shapes import (
    array, blob, closed, enum, identifier, ids, require, text, unique, version,
)


ARTIFACT_KIND = "datum.workflow-delivery.artifacts/v1"


def validate_environment(tree, proof, requested, *, contract=None):
    require(requested is not None, "environment", "explicit requested environment required",
            "WDQ-ENVIRONMENT")
    recorded = parse_json(read_blob(tree, proof["environment"], code="WDQ-ENVIRONMENT"),
                          proof["environment"]["path"])
    if any(type(v) is dict and "schema_version" in v for v in (recorded, requested)):
        receipt = parse_json(read_blob(tree, proof["build"]["receipt"], code="WDQ-ENVIRONMENT"),
                             proof["build"]["receipt"]["path"])
        validate_headless(recorded, requested,
                          category=contract.get("category") if contract else None,
                          toolchain=proof["build"]["toolchain"],
                          binary_sha256=receipt["binary_sha256"])
        return
    for value in (recorded, requested):
        closed(value, "os backend toolchain scale input_method window_size reproduction_commands",
               "environment", "WDQ-ENVIRONMENT")
        for key in ("os", "backend", "toolchain", "input_method"):
            text(value[key], "environment." + key)
        require(type(value["scale"]) in (int, float) and value["scale"] > 0,
                "environment.scale", "positive scale required", "WDQ-ENVIRONMENT")
        size = value["window_size"]
        require(type(size) is list and len(size) == 2
                and all(type(n) is int and n > 0 for n in size), "environment.window_size",
                "positive integer width/height required", "WDQ-ENVIRONMENT")
        array(value["reproduction_commands"], "environment.reproduction_commands")
        for command in value["reproduction_commands"]:
            text(command, "environment.reproduction_commands")
    require(recorded == requested, proof["environment"]["path"],
            "requested environment differs; no equivalence is implicitly approved",
            "WDQ-ENVIRONMENT")
    require(recorded["toolchain"] == proof["build"]["toolchain"], "environment.toolchain",
            "environment/build toolchain mismatch", "WDQ-ENVIRONMENT")


def _json_artifact(tree, item):
    return parse_json(read_blob(tree, item), item["path"])


def _index(tree, result):
    found = []
    for item in result["artifacts"]:
        raw = read_blob(tree, item)
        # Only a JSON object explicitly identifying this versioned protocol is an
        # index. Images and arbitrary text are never guessed to be event records.
        if raw.lstrip().startswith(b"{"):
            value = parse_json(raw, item["path"])
            if type(value) is dict and value.get("kind") == ARTIFACT_KIND:
                found.append((item, value))
    require(len(found) == 1, result["scenario_id"],
            "exactly one typed artifact-role index required", "WDQ-RESULT")
    index_blob, value = found[0]
    closed(value, "kind scenario_id events captures state registry", index_blob["path"])
    require(value["scenario_id"] == result["scenario_id"], index_blob["path"],
            "artifact index scenario mismatch", "WDQ-RESULT")
    members = []
    for role in ("events", "captures", "state"):
        array(value[role], index_blob["path"] + "." + role, nonempty=role != "captures")
        for member in value[role]:
            blob(member, index_blob["path"])
            members.append(member)
    if value["registry"] is not None:
        blob(value["registry"], index_blob["path"])
        members.append(value["registry"])
    unique([m["path"] for m in members], index_blob["path"])
    expected = sorted(result["artifacts"], key=lambda x: x["path"])
    require(sorted(members + [index_blob], key=lambda x: x["path"]) == expected,
            index_blob["path"], "roles must bind every sibling artifact exactly", "WDQ-RESULT")
    return value


def _registry(tree, item, binary, contract):
    require(item is not None, "registry", "production registry export required", "WDQ-CONSUMER")
    value = _json_artifact(tree, item)
    closed(value, "schema_version binary_sha256 entries", item["path"])
    version(value["schema_version"], item["path"])
    require(value["binary_sha256"] == binary, item["path"],
            "registry must identify tested binary", "WDQ-CONSUMER")
    array(value["entries"], item["path"])
    for entry in value["entries"]:
        closed(entry, "dispatch_key handler_ref entry_surfaces contexts", item["path"])
        text(entry["dispatch_key"], item["path"])
        ids(entry["entry_surfaces"], item["path"])
        require(type(entry["contexts"]) is dict and bool(entry["contexts"]), item["path"],
                "production contexts required", "WDQ-CONSUMER")
        for context, state in entry["contexts"].items():
            identifier(context, item["path"])
            closed(state, "enabled reason", item["path"])
            require(type(state["enabled"]) is bool, item["path"], "boolean eligibility required")
            text(state["reason"], item["path"])
            require(not state["enabled"] or entry["handler_ref"] is not None,
                    item["path"], "enabled consumer has no production handler", "WDQ-CONSUMER")
    unique([e["dispatch_key"] for e in value["entries"]], item["path"])
    entries = {e["dispatch_key"]: e for e in value["entries"]}
    for consumer in contract["consumers"]:
        key = consumer["dispatch_key"]
        require(key in entries, item["path"], f"production key absent: {key}", "WDQ-CONSUMER")
        entry = entries[key]
        require(entry["handler_ref"] == consumer["handler_ref"]
                and set(entry["entry_surfaces"]) == set(consumer["entry_surfaces"]),
                item["path"], f"production mapping disagrees: {key}", "WDQ-CONSUMER")
    return entries


def validate_correlations(tree, contract, proof):
    receipt = _json_artifact(tree, proof["build"]["receipt"])
    binary = receipt["binary_sha256"]
    authority = authority_sha256(tree, contract)
    event_hashes = set()
    seen_surfaces = {c["id"]: set() for c in contract["consumers"]}
    consumers = {c["id"]: c for c in contract["consumers"]}
    scenarios = {s["id"]: s for s in contract["scenarios"]}
    for result in proof["results"]:
        scenario = scenarios[result["scenario_id"]]
        roles = _index(tree, result)
        if contract["category"] == "product":
            require(bool(roles["captures"]), scenario["id"], "native capture required", "WDQ-RESULT")
        entries = _registry(tree, roles["registry"], binary, contract)
        for event_blob in roles["events"]:
            event_hashes.add(event_blob["sha256"])
            record = _json_artifact(tree, event_blob)
            closed(record, "schema_version scenario_id producer_session method binary_sha256 authority_sha256 "
                   "inputs dispatches actual_visible actual_state", event_blob["path"])
            version(record["schema_version"], event_blob["path"])
            require(record["authority_sha256"] == authority, event_blob["path"],
                    "governing authority changed since observed run", "WDQ-STALE")
            require(record["scenario_id"] == scenario["id"]
                    and record["producer_session"] == proof["producer_session"]
                    and record["binary_sha256"] == binary
                    and record["method"] == scenario["method"]
                    and record["inputs"] == scenario["inputs"], event_blob["path"],
                    "input/method/session/binary correlation mismatch", "WDQ-RESULT")
            require(record["actual_visible"] == result["actual_visible"]
                    and record["actual_state"] == result["actual_state"], event_blob["path"],
                    "event and observed result disagree", "WDQ-RESULT")
            array(record["dispatches"], event_blob["path"])
            for event in record["dispatches"]:
                known = _dispatch(event, entries, event_blob["path"])
                if not known:
                    continue  # Observed fail-closed fallback, not a supported consumer.
                matches = [cid for cid in scenario["consumer_ids"]
                           if consumers[cid]["dispatch_key"] == event["dispatch_key"]]
                require(bool(matches), event_blob["path"], "unreviewed scenario consumer", "WDQ-CONSUMER")
                for cid in matches:
                    seen_surfaces[cid].add(event["entry_surface"])
    for cid, surfaces in seen_surfaces.items():
        require(surfaces == set(consumers[cid]["entry_surfaces"]), cid,
                "all reviewed entry surfaces require dispatch evidence", "WDQ-CONSUMER")
    return event_hashes


def event_identities(tree, proof):
    """Only explicitly typed event roles participate in the copied-replay test."""
    return {event["sha256"] for result in proof["results"]
            for event in _index(tree, result)["events"]}


def _dispatch(event, entries, path):
    closed(event, "dispatch_key entry_surface context eligible enabled invoked handler_ref "
           "unavailable_reason mutation_count", path)
    key = event["dispatch_key"]
    text(key, path)
    for name in ("eligible", "enabled", "invoked"):
        require(type(event[name]) is bool, path, f"{name} must be boolean", "WDQ-CONSUMER")
    require(type(event["mutation_count"]) is int and event["mutation_count"] >= 0,
            path, "nonnegative mutation count required", "WDQ-CONSUMER")
    if key not in entries:
        require(not event["eligible"] and not event["enabled"] and not event["invoked"]
                and event["handler_ref"] is None and event["mutation_count"] == 0
                and type(event["unavailable_reason"]) is str and bool(event["unavailable_reason"].strip()),
                path, "unknown key must be refused without invocation/mutation", "WDQ-CONSUMER")
        return False
    entry = entries[key]
    context = event["context"]
    require(type(context) is str and context in entry["contexts"]
            and event["entry_surface"] in entry["entry_surfaces"], path,
            "unknown context or entry surface", "WDQ-CONSUMER")
    expected = entry["contexts"][context]
    require(event["eligible"] == event["enabled"] == expected["enabled"]
            and event["handler_ref"] == entry["handler_ref"], path,
            "eligibility/enablement/dispatch registry disagreement", "WDQ-CONSUMER")
    if not event["enabled"]:
        require(not event["invoked"] and event["mutation_count"] == 0
                and event["unavailable_reason"] == expected["reason"], path,
                "unavailable path invoked/mutated or concealed its reason", "WDQ-CONSUMER")
    else:
        require(event["invoked"] and event["handler_ref"] is not None, path,
                "enabled activation did not reach handler", "WDQ-CONSUMER")
    return True
