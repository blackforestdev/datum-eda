"""Qualify current diagnostic correlation against complete public selection output.

The expected IDs/revision must come from a frozen fixture or an independent
authoritative query. Compact diagnostics alone never qualify membership.
"""
from __future__ import annotations

import hashlib
import json

PREFIX = "DATUM_ACTION_EVIDENCE "
KIND = "datum.action-evidence/v2"


def latest_state(text):
    latest, sequence = None, 0
    for line in text.splitlines(keepends=True):
        if not line.startswith(PREFIX) or not line.endswith("\n"):
            continue  # An in-flight final write is not yet an observation.
        record = json.loads(line[len(PREFIX):])
        if record.get("kind") != KIND or record.get("event") == "diagnostics_incomplete":
            raise ValueError("unsupported or incomplete diagnostic stream")
        if not isinstance(record.get("sequence"), int) or record["sequence"] <= sequence:
            raise ValueError("diagnostic sequence is not increasing")
        sequence = record["sequence"]
        if record["event"] == "state":
            latest = record
    if latest is None:
        raise ValueError("no complete current state observation")
    return latest


def identity(value):
    if (not isinstance(value, dict) or set(value) - {"class", "id", "instance_path"}
            or not isinstance(value.get("class"), str) or not isinstance(value.get("id"), str)
            or not isinstance(value.get("instance_path", []), list)
            or not all(isinstance(p, str) for p in value.get("instance_path", []))):
        raise ValueError("incomplete typed source identity")
    return value["class"], value["id"], tuple(value.get("instance_path", []))


def qualify(record, snapshot, expected):
    """Reject incomplete, duplicate, stale or foreign observations before credit."""
    if record.get("kind") != KIND or record.get("event") != "state":
        raise ValueError("current state record required")
    selection = record["value"]["selection"]
    if selection.get("stale") is not False:
        raise ValueError("explicit current non-stale selection required")
    if selection.get("complete_inspection") is not False:
        raise ValueError("expected current compact diagnostic correlation")
    if snapshot.get("pid") != expected["pid"]:
        raise ValueError("wrong observed process")
    basis = {"project-id": expected["project"], "model-revision": expected["revision"],
             "subject-kind": expected["kind"]}
    if snapshot["before"] != snapshot["after"]:
        raise ValueError("selection changed during complete public read")
    for attrs in [snapshot["before"], snapshot["after"]]:
        if any(attrs.get(k) != v for k, v in basis.items()):
            raise ValueError("wrong public project/revision/subject")
    if any(selection.get(k) != expected[k] for k in ["project", "revision", "kind"]):
        raise ValueError("wrong diagnostic project/revision/subject")
    if selection.get("scope") != expected.get("scope"):
        raise ValueError("wrong inspection scope")
    members, signatures = [], []
    for member in snapshot["members"]:
        attrs = member["attributes"]
        if (any(attrs.get(k) != v for k, v in basis.items())
                or attrs.get("selected") != "true" or member.get("standard_selected") is not True):
            raise ValueError("foreign, stale or unselected public member")
        value = json.loads(attrs["source-identity"])
        signature = identity(value)
        canonical = {"class": signature[0], "id": signature[1]}
        if signature[2]:
            canonical["instance_path"] = list(signature[2])
        members.append(canonical)
        signatures.append(signature)
    wanted = [identity(value) for value in expected["members"]]
    if len(set(signatures)) != len(signatures) or len(set(wanted)) != len(wanted):
        raise ValueError("duplicate observed or expected identity")
    if set(signatures) != set(wanted):
        raise ValueError("complete public membership differs from independent oracle")
    if snapshot["count"] != len(members) or selection.get("member_count") != len(members):
        raise ValueError("missing complete public members")
    digest = hashlib.sha256(json.dumps(members, separators=(",", ":"),
                                       ensure_ascii=False).encode()).hexdigest()
    if 'sha256:' + digest != selection.get("members_sha256"):
        raise ValueError("diagnostic correlation differs from complete public membership")
    return {"sequence": record["sequence"], "pid": expected["pid"], **basis,
            "members": members, "compact_correlated": True,
            "oracle": expected["oracle"], "public_snapshot": snapshot}
