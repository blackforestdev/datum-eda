"""Malformed synthetic coverage INPUT, never real roadmap reclassification."""

from copy import deepcopy
from datetime import datetime, timedelta, timezone

from workflow_delivery_io import canonical_json, parse_json, sha256

BOUNDARY_CASES = {
    "INFRA-S05-06.delete-unpermitted": ("external/unpermitted/deletable.py", "WDQ-COVERAGE", "production change has no promoted scope"),
    "INFRA-S05-06.rename-escape": ("external/unpermitted/renamed.py", "WDQ-COVERAGE", "production change has no promoted scope"),
    "INFRA-S02-11.rust": ("crates/synthetic/src/new.rs", "WDQ-COVERAGE", "production change has no promoted scope"),
    "INFRA-S02-11.mcp": ("mcp-server/synthetic_new.py", "WDQ-COVERAGE", "production change has no promoted scope"),
    "INFRA-S02-11.workflow": ("scripts/synthetic_workflow.py", "WDQ-COVERAGE", "production change has no promoted scope"),
    "INFRA-S02-02.reopen": ("TASK", "WDQ-COVERAGE", "historical reopening requires reclassification"),
    "INFRA-S02-02.add-execution": ("TASK", "WDQ-COVERAGE", "historical completion/landing evidence changed"),
    "INFRA-S02-04.retained-history": (None, None, None),
    "INFRA-S02-04.reactivate": ("COLD", "WDQ-COVERAGE", "deferred work requires reclassification"),
    "INFRA-S02-04.rewrite-history": ("COLD", "WDQ-COVERAGE", "deferred work requires reclassification"),
    "INFRA-S02-05.authorization": ("NEXT", "WDQ-COVERAGE", "external-lane authorization changed"),
    "INFRA-S02-05.selected-step": ("NEXT", "WDQ-COVERAGE", "external-lane completion boundary changed"),
    "INFRA-S02-05.requirement": ("NEXT", "WDQ-COVERAGE", "external-lane selected requirement/authority changed"),
    "INFRA-S02-05.future-requirement": ("NEXT", "WDQ-COVERAGE", "external-lane work crossed the promoted boundary"),
    "INFRA-S02-06.authorized": (None, None, None),
    "INFRA-S02-06.unpermitted": ("external/unpermitted/input.py", "WDQ-COVERAGE", "production change has no promoted scope"),
    "INFRA-S02-09.prefix-lookalike": ("external/owned_elsewhere/input.py", "WDQ-COVERAGE", "production change has no promoted scope"),
    "INFRA-S02-08.absent": ("specs/active_frontier.json", "WDQ-TRANSITION", "in_progress item requires a claim object"),
    "INFRA-S02-08.expired": ("specs/active_frontier.json", "WDQ-TRANSITION", "claim expired at"),
    "INFRA-S02-08.mismatched": ("specs/active_frontier.json", "WDQ-TRANSITION", "does not match claim agent"),
}

COVERAGE_INPUT_CASES = {
    "INFRA-S02-01.newly-unclassified": (
        "specs/active_frontier.json", "WDQ-COVERAGE", "classification mismatch: missing=['NEW'], absent=[]"),
    "INFRA-S02-01.missing-classification": (
        "specs/active_frontier.json", "WDQ-COVERAGE", "classification mismatch: missing=['NEXT'], absent=[]"),
    "INFRA-S02-01.duplicate-classification": (
        "specs/workflow_delivery_policy.json", "WDQ-TRUST", "selected authority has no valid promoted policy"),
}
COVERAGE_INPUT_CASES.update(BOUNDARY_CASES)


def coverage_expectation(variant, surface):
    if variant.startswith("INFRA-S02-08.") and surface.startswith("S-"):
        return None, "NEXT", BOUNDARY_CASES[variant][2]
    if variant == "INFRA-S02-01.duplicate-classification" and surface.startswith("S-"):
        return "coverage.rows", "WDQ-POLICY", "duplicate identity"
    return COVERAGE_INPUT_CASES[variant]


def prepare_coverage_input(policy, variant, *, fixture=None, manifest=None):
    if variant not in COVERAGE_INPUT_CASES:
        raise ValueError("explicit implemented coverage-input variant required")
    if variant in BOUNDARY_CASES:
        prepare_external_boundary(fixture, manifest, policy)
        return
    rows = policy["coverage"]["rows"]
    if variant.endswith(".newly-unclassified"):
        from project_status import render_block
        item = deepcopy(manifest["frontier"][1])
        item.update(key="NEW", issue_id="dat-new", order=2, canonical_next=False)
        item["completion"]["canonical_next_step_id"] = "NEW-C01"
        step = item["completion"]["steps"][0]
        step["id"] = "NEW-C01"
        step["requirement_refs"][0]["marker"] = "NEW-C01"
        manifest["frontier"].append(item)
        doc = item["governing_docs"][0]
        fixture.write(doc, (fixture.root / doc).read_bytes() + b"\n<!-- REQ:NEW:NEW-C01 -->\n")
        fixture.write(".beads/issues.jsonl", (fixture.root / ".beads/issues.jsonl").read_bytes() +
            canonical_json({"id": "dat-new", "status": "open", "labels": ["roadmap:frontier"],
                            "acceptance_criteria": "NEW-C01: Newly scheduled synthetic planning"}))
        fixture.save("specs/active_frontier.json", manifest)
        fixture.write("specs/PROGRESS.md", render_block(manifest).encode())
        # Existing TASK/NEXT classifications remain intact; only NEW is absent.
    elif variant.endswith(".missing-classification"):
        policy["coverage"]["rows"] = [row for row in rows if row["frontier_key"] != "NEXT"]
    else:
        rows.append(deepcopy(rows[0]))


def prepare_external_boundary(f, manifest, policy):
    """Freeze a synchronized synthetic external lane, not real Preferences."""
    from project_status import render_block
    item = manifest["frontier"][1]
    item["completion"]["steps"][0].update(kind="execution", status="in_progress")
    second = deepcopy(item["completion"]["steps"][0])
    second.update(id="NEXT-C02", status="pending", depends_on=["NEXT-C01"])
    doc = item["governing_docs"][0]
    second["requirement_refs"] = [{"path": doc, "marker": "NEXT-C02"}]
    item["completion"]["steps"].append(second)
    f.write(doc, (f.root / doc).read_bytes() + b"\n<!-- REQ:NEXT:NEXT-C02 -->\n")
    now = datetime.now(timezone.utc).replace(microsecond=0) - timedelta(seconds=1)
    stamp = lambda value: value.isoformat().replace("+00:00", "Z")
    item.update(state="in_progress", authorization="execution", claim={
        "agent": "codex", "harness": "codex-cli", "session": "external-capture-fixture",
        "worktree": str(f.root), "head": f.git("rev-parse", "HEAD").decode().strip(),
        "scope": ["external/owned"], "claimed_at": stamp(now), "heartbeat_at": stamp(now),
        "expires_at": stamp(now + timedelta(hours=1))})
    issues = [parse_json(line, ".beads/issues.jsonl") for line in
              (f.root / ".beads/issues.jsonl").read_bytes().splitlines() if line.strip()]
    next(issue for issue in issues if issue["id"] == "dat-next").update(status="in_progress", assignee="codex",
        acceptance_criteria="NEXT-C01: Current bounded work\nNEXT-C02: Later separately authorized work")
    cold = deepcopy(item)
    cold.update(key="COLD", issue_id="dat-cold", order=2, state="deferred",
                authorization="none", canonical_next=False)
    del cold["claim"]
    previous, future = cold["completion"]["steps"]
    previous.update(id="COLD-C01", status="complete",
                    completion_evidence=[{"kind": "document", **f.ref}],
                    requirement_refs=[{"path": doc, "marker": "COLD-C01"}])
    future.update(id="COLD-C02", status="pending", depends_on=["COLD-C01"],
                  requirement_refs=[{"path": doc, "marker": "COLD-C02"}])
    cold["completion"]["canonical_next_step_id"] = "COLD-C02"
    manifest["frontier"].append(cold)
    f.write(doc, (f.root / doc).read_bytes() +
            b"\n<!-- REQ:COLD:COLD-C01 -->\n<!-- REQ:COLD:COLD-C02 -->\n")
    issues.append({"id": "dat-cold", "status": "deferred", "labels": ["roadmap:deferred"],
                   "acceptance_criteria": "COLD-C01: Retained history\nCOLD-C02: Deferred future work"})
    policy["coverage"]["rows"].append({"frontier_key": "COLD", "issue_id": "dat-cold",
        "category": "deferred", "boundary_ref": f.ref, "external_handoff_ref": None})
    f.write(".beads/issues.jsonl", b"".join(canonical_json(issue) for issue in issues))
    policy["coverage"]["rows"][1].update(category="external_lane", boundary_ref=f.ref, external_handoff_ref=f.ref)
    policy["coverage"]["production_roots"].extend(["external", "crates", "mcp-server", "scripts"])
    policy["coverage"]["source_scopes"] = [{"frontier_key": "NEXT", "step_ids": ["NEXT-C01"],
        "paths": ["external/owned"], "boundary_ref": f.ref}]
    f.write("external/owned/input.py", b"# Synthetic external baseline.\n")
    f.write("external/unpermitted/deletable.py", b"# Retained unowned synthetic input.\n")
    f.save("specs/active_frontier.json", manifest)
    f.write("specs/PROGRESS.md", render_block(manifest).encode())


def mutate_external_boundary(root, case_id):
    """Make explicit candidate changes after the exact synthetic authority pin."""
    from project_status import render_block
    from workflow_delivery_capture_state import git
    changes = {}
    path = (BOUNDARY_CASES[case_id][0] if case_id.startswith("INFRA-S02-11.") else
            "external/unpermitted/input.py" if case_id.endswith(".unpermitted") else
            "external/owned_elsewhere/input.py" if case_id.endswith(".prefix-lookalike") else
            "external/owned/input.py")
    changes[path] = (b"// Actual synthetic Rust source, never compiled.\npub fn fixture() {}\n"
                     if path.endswith(".rs") else b"# Actual candidate change in an isolated synthetic lane.\n")
    if case_id.endswith(".delete-unpermitted"):
        changes["external/unpermitted/deletable.py"] = None
    elif case_id.endswith(".rename-escape"):
        changes["external/unpermitted/renamed.py"] = (root / "external/owned/input.py").read_bytes()
        changes["external/owned/input.py"] = None
    if case_id.startswith("INFRA-S02-02."):
        manifest = parse_json((root / "specs/active_frontier.json").read_bytes(), "Frontier")
        historical = next(item for item in manifest["frontier"] if item["key"] == "TASK")
        issues = [parse_json(line, "beads") for line in (root / ".beads/issues.jsonl").read_bytes().splitlines()]
        issue = next(issue for issue in issues if issue["id"] == "dat-test")
        if case_id.endswith(".reopen"):
            historical["state"] = "ready"
            del historical["landing_commit"]
            issue["status"] = "open"
        else:
            extra = deepcopy(next(step for step in historical["completion"]["steps"] if step["id"] == "V"))
            extra.update(id="EXTRA", action="Candidate-only additional historical execution",
                         depends_on=["A"], requirement_refs=[{"path": historical["governing_docs"][0], "marker": "EXTRA"}])
            historical["completion"]["steps"].append(extra)
            doc = historical["governing_docs"][0]
            changes[doc] = (root / doc).read_bytes() + b"\n<!-- REQ:TASK:EXTRA -->\n"
            issue["acceptance_criteria"] += "\nEXTRA: Additional historical execution"
        changes[".beads/issues.jsonl"] = b"".join(canonical_json(issue) for issue in issues)
        changes["specs/active_frontier.json"] = canonical_json(manifest)
        changes["specs/PROGRESS.md"] = render_block(manifest).encode()
    if case_id.startswith("INFRA-S02-04.") and not case_id.endswith(".retained-history"):
        manifest = parse_json((root / "specs/active_frontier.json").read_bytes(), "Frontier")
        cold = next(item for item in manifest["frontier"] if item["key"] == "COLD")
        if case_id.endswith(".reactivate"):
            cold.update(state="ready", authorization="execution")
            issues = [parse_json(line, "beads") for line in (root / ".beads/issues.jsonl").read_bytes().splitlines()]
            next(issue for issue in issues if issue["id"] == "dat-cold").update(
                status="open", labels=["roadmap:frontier"])
            changes[".beads/issues.jsonl"] = b"".join(canonical_json(issue) for issue in issues)
        else:
            cold["completion"]["steps"][0]["action"] = "Rewritten completed historical obligation"
        changes["specs/active_frontier.json"] = canonical_json(manifest)
        changes["specs/PROGRESS.md"] = render_block(manifest).encode()
    if case_id.startswith(("INFRA-S02-08.", "INFRA-S02-05.")):
        manifest = parse_json((root / "specs/active_frontier.json").read_bytes(), "Frontier")
        item = manifest["frontier"][1]
        first, second = item["completion"]["steps"]
        if case_id.endswith(".authorization"):
            # Keep the scoped step an execution step so source-scope shape
            # remains valid. A coherent planning successor must still fail
            # the earlier exact external-authorization boundary.
            item["authorization"] = "planning"
            first.update(status="complete", completion_evidence=[{"kind": "document",
                "path": "docs/authority.md", "marker": "<!-- RULE -->"}])
            second.update(kind="planning", status="in_progress")
            item["completion"]["canonical_next_step_id"] = "NEXT-C02"
        elif case_id.endswith(".selected-step"):
            first.update(status="complete", completion_evidence=[{"kind": "document",
                "path": "docs/authority.md", "marker": "<!-- RULE -->"}])
            second["status"] = "in_progress"
            item["completion"]["canonical_next_step_id"] = "NEXT-C02"
        elif case_id.endswith(".requirement"):
            first["action"] = "Expanded unauthorized current action"
        elif case_id.endswith(".future-requirement"):
            second["action"] = "Expanded unauthorized future action"
        elif case_id.endswith(".absent"):
            del item["claim"]
        elif case_id.endswith(".expired"):
            for field in ("claimed_at", "heartbeat_at", "expires_at"):
                value = datetime.fromisoformat(item["claim"][field].replace("Z", "+00:00"))
                item["claim"][field] = (value - timedelta(hours=2)).isoformat().replace("+00:00", "Z")
        else:
            issues = [parse_json(line, "beads") for line in (root / ".beads/issues.jsonl").read_bytes().splitlines()]
            next(issue for issue in issues if issue["id"] == "dat-next")["assignee"] = "different-owner"
            changes[".beads/issues.jsonl"] = b"".join(canonical_json(issue) for issue in issues)
        changes["specs/active_frontier.json"] = canonical_json(manifest)
        changes["specs/PROGRESS.md"] = render_block(manifest).encode()
    records = []
    for name, raw in sorted(changes.items()):
        target = root / name
        records.append({"path": name, "before_sha256": sha256(target.read_bytes()) if target.exists() else None,
                        "after_hex": raw.hex() if raw is not None else None})
        if raw is None:
            target.unlink()  # Exact fixture-owned source deletion, retained in its bundle.
        else:
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(raw)
    git(root, "add", "--", *sorted(changes))
    return {"case_id": case_id, "changes": records, "synthetic_input_only": True,
            "staged_name_status": git(root, "diff", "--cached", "--name-status", "--find-renames=100%", "--").decode()}
