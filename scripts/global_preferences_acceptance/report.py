"""Compute V2 readiness from complete observations; acceptance is separately selected."""

from .identity import command_line, validate_candidate, validate_environments
from .inputs import Bundle, array, canonical, closed, digest, parse, read_file, require, revision, sha, text, validation
from .inventory import PRODUCT_BOUNDARY
from .matrix import validate_matrix
from .measurements import measurement_inventory, validate_trials
from .observations import identity, validate_cases, validate_gates, validate_native


def review_subject(report):
    """All proof roots include transitive artifact hashes; exclude the review cycle."""
    return digest(canonical({k: v for k, v in report.items()
                             if k not in {"artifacts", "independent_review"}}))


@validation
def validate_report(report, matrix, *, root, bundle_root, expected_revision, expected_environment_sha256,
                    test_only=False, review_only=False):
    closed(report, "schema origin candidate matrix_sha256 input_manifest environments artifacts case_results "
           "measurements gate_results native_observations independent_review release_notes", "report")
    require(report["schema"] == "datum-global-preferences-production-evidence-v2",
            "historical evidence cannot establish V2 readiness")
    validate_matrix(matrix)
    closed(report["origin"], "kind producer_session", "capture origin")
    text(report["origin"]["producer_session"], "producing session")
    require(report["origin"]["kind"] == ("synthetic-test" if test_only else "product-execution"),
            "synthetic evidence is not eligible for production readiness")
    require(canonical(matrix) == canonical(parse(read_file(root,
            "specs/global_preferences_production_acceptance_matrix.json"), "committed matrix")),
            "supplied matrix differs from actual proof input")
    require(digest(canonical(report["environments"])) == sha(expected_environment_sha256, "selected environments"),
            "observed environments differ from the selected reference environment record")
    require(sha(report["matrix_sha256"], "matrix digest") == digest(canonical(matrix)),
            "report matrix digest differs")
    bundle = Bundle(bundle_root, report["artifacts"])
    binaries = validate_candidate(root, report["candidate"], report["input_manifest"],
                                  bundle, expected_revision)
    environments = validate_environments(report["environments"])
    validate_cases(report, bundle, environments, binaries)
    validate_native(report, bundle, environments, binaries)
    expected = measurement_inventory()
    require(len(array(report["measurements"], "measurements")) == len(expected), "measurement count differs")
    seen = set()
    for row in array(report["measurements"], "measurements"):
        closed(row, "budget_id variant_id environment_id candidate_revision binary_sha256 "
               "input_manifest_sha256 matrix_sha256 trials", "measurement")
        coordinate = identity(row, report, environments, binaries)
        key = (row["budget_id"], row["variant_id"], coordinate)
        require(key in expected and key not in seen, "unknown or duplicate measurement variant")
        seen.add(key)
        validate_trials(row, bundle)
    require(seen == expected, f"missing measurement variants: {len(expected - seen)}")
    validate_gates(report, bundle, root)
    if not review_only:
        validate_review(bundle.json(report["independent_review"]), report, bundle)
    validate_release(bundle.json(report["release_notes"]), report, bundle)
    return {"state": "synthetic_validated" if test_only else "reviewable" if review_only else "candidate_ready",
            "candidate": expected_revision, "input_manifest_sha256": report["input_manifest"]["sha256"],
            "matrix_sha256": report["matrix_sha256"],
            "review_subject_sha256": review_subject(report),
            "report_sha256": digest(canonical(report)), "production_accepted": False}


def validate_review(value, report, bundle):
    closed(value, "schema producer_session reviewer_session review_subject_sha256 candidate_revision input_manifest_sha256 "
           "matrix_sha256 replay finding_dispositions exclusions", "independent review")
    require(value["schema"] == "datum.preferences.independent-review.v2", "unknown review schema")
    for name in ("producer_session", "reviewer_session"):
        text(value[name], name)
    require(value["producer_session"] != value["reviewer_session"], "review session is not independent")
    require(value["producer_session"] == report["origin"]["producer_session"], "review producer differs")
    require(value["review_subject_sha256"] == review_subject(report), "independent review selects different evidence")
    for key, expected in (("candidate_revision", report["candidate"]["revision"]),
                          ("input_manifest_sha256", report["input_manifest"]["sha256"]),
                          ("matrix_sha256", report["matrix_sha256"])):
        require(value[key] == expected, "independent review identity differs")
    seen = set()
    for replay in array(value["replay"], "independent replay"):
        closed(replay, "category command exit_code log coverage", "independent replay row")
        require(replay["category"] in {"evidence", "durability", "security", "native"}
                and replay["category"] not in seen, "duplicate/unknown independent replay category")
        seen.add(replay["category"])
        command = command_line(replay["command"])
        require(type(replay["exit_code"]) is int and replay["exit_code"] == 0, "independent replay failed")
        from .observations import command_observation
        output = command_observation({**replay, "candidate_revision": value["candidate_revision"],
                                      "input_manifest_sha256": value["input_manifest_sha256"]}, bundle)
        category = replay["category"]
        coverage = array(replay["coverage"], "independent replay coverage")
        if category == "evidence":
            require(command[:3] == ["python3", "scripts/check_global_preferences_production_evidence.py", "--review"]
                    and len(command) == 11, "independent evidence replay command differs")
            flags = dict(zip(command[3::2], command[4::2]))
            require(set(flags) == {"--report", "--bundle-root", "--candidate", "--environment-sha256"}
                    and flags["--candidate"] == value["candidate_revision"]
                    and flags["--environment-sha256"] == digest(canonical(report["environments"]))
                    and coverage == ["complete-report"], "independent evidence replay inputs differ")
            result = parse(output, "independent evidence replay result")
            closed(result, "state candidate input_manifest_sha256 matrix_sha256 review_subject_sha256 "
                   "report_sha256 production_accepted", "independent evidence replay verdict")
            require(result.get("state") == "reviewable" and result.get("candidate") == value["candidate_revision"]
                    and result.get("input_manifest_sha256") == value["input_manifest_sha256"]
                    and result.get("matrix_sha256") == value["matrix_sha256"]
                    and result["review_subject_sha256"] == value["review_subject_sha256"]
                    and result["production_accepted"] is False, "independent evidence replay failed or stale")
        elif category in {"durability", "security"}:
            families = {"project-genesis-crash", "project-genesis-replay", "backup-restore"} if category == "durability" else {
                "human-agent-authority", "surface-parity-negative"}
            matches = [r for r in report["case_results"] if r["case_id"] in families and r["command"] == command]
            require(bool(matches) and coverage == [matches[0]["case_id"], matches[0]["variant_id"],
                    matches[0]["subcase_id"], matches[0]["surface"], matches[0]["environment_id"]],
                    "independent replay did not execute a required case")
            command_observation({**matches[0], "log": replay["log"]}, bundle, tests=matches[0]["executed_tests"])
        else:
            matches = [r for r in report["native_observations"] if coverage == [r["scenario_id"], r["environment_id"]]]
            require(len(matches) == 1, "independent native replay coverage differs")
            selected = matches[0]
            recipe = bundle.json(selected["fixture"])["recipe"]["path"]
            require(command == ["python3", recipe, "--native-scenario", selected["scenario_id"],
                                "--environment", selected["environment_id"]], "independent native replay command differs")
            result = parse(output, "independent native replay output")
            require(canonical(result) == canonical({"schema": "datum.preferences.native-replay.v2",
                    "candidate_revision": value["candidate_revision"], "input_manifest_sha256": value["input_manifest_sha256"],
                    "matrix_sha256": value["matrix_sha256"], "coverage": coverage,
                    "capture": selected["capture"], "accessibility_capture": selected["accessibility_capture"]}),
                    "independent native replay did not verify the selected captures")
    require(seen == {"evidence", "durability", "security", "native"}, "independent replay incomplete")
    ids = set()
    for finding in array(value["finding_dispositions"], "review findings", nonempty=False):
        closed(finding, "id disposition evidence", "review finding")
        text(finding["id"], "finding id")
        require(finding["id"] not in ids and finding["disposition"] == "resolved", "unresolved/waived finding")
        ids.add(finding["id"])
        require(bool(bundle.read(finding["evidence"])), "finding disposition evidence missing")
    require(value["exclusions"] == PRODUCT_BOUNDARY["excluded"], "independent exclusions incomplete")


def validate_release(value, report, bundle):
    closed(value, "schema candidate_revision input_manifest_sha256 environment_ids binaries "
           "boundaries support clean_user_launch", "release handoff")
    require(value["schema"] == "datum.preferences.release-handoff.v2" and
            value["candidate_revision"] == report["candidate"]["revision"] and
            value["input_manifest_sha256"] == report["input_manifest"]["sha256"], "release identity differs")
    require(value["environment_ids"] == [e["id"] for e in report["environments"]], "release environments differ")
    require(value["binaries"] == report["candidate"]["binaries"], "distributed binary identities differ")
    require(canonical(value["boundaries"]) == canonical(PRODUCT_BOUNDARY), "release boundary differs")
    closed(value["support"], "settings_locations diagnostics errors recovery compatibility storage "
           "retained_history known_exclusions", "support packet")
    for reference in value["support"].values():
        require(bool(bundle.read(reference)), "support instructions absent")
    require(bool(bundle.read(value["clean_user_launch"])), "clean-user offline launch proof absent")


@validation
def validate_acceptance(receipt_raw, *, expected_receipt_sha256, report, matrix, root, bundle_root,
                        expected_revision, expected_environment_sha256):
    """An external exact receipt selection is mandatory; report flags grant nothing."""
    require(digest(receipt_raw) == sha(expected_receipt_sha256, "externally selected receipt digest"),
            "owner receipt bytes differ")
    readiness = validate_report(report, matrix, root=root, bundle_root=bundle_root,
                                expected_revision=expected_revision,
                                expected_environment_sha256=expected_environment_sha256)
    receipt = parse(receipt_raw, "owner receipt")
    closed(receipt, "schema candidate_revision report_sha256 disposition response provenance publication_delta", "owner receipt")
    require(receipt["schema"] == "datum.preferences.owner-acceptance.v2", "unknown owner receipt")
    require(readiness["state"] == "candidate_ready", "readiness must pass before acceptance")
    require(receipt["candidate_revision"] == report["candidate"]["revision"] and
            receipt["report_sha256"] == digest(canonical(report)), "owner receipt selects other evidence")
    require(receipt["disposition"] == "accept", "owner did not accept")
    expected = "GLOBAL-PREFERENCES-COMPLETION: accept GP-CM05E " + receipt["candidate_revision"] + " " + receipt["report_sha256"]
    require(receipt["response"] == expected, "owner response does not select the exact candidate/report")
    closed(receipt["provenance"], "source recorded_at", "owner provenance")
    for field in receipt["provenance"].values():
        text(field, "owner provenance")
    closed(receipt["publication_delta"], "candidate publication changed_paths review", "publication delta")
    delta = receipt["publication_delta"]
    require(delta["candidate"] == receipt["candidate_revision"], "publication base differs")
    publication = revision(delta["publication"])
    from .identity import git
    require(git(root, "merge-base", delta["candidate"], publication).decode().strip() == delta["candidate"],
            "publication is not a descendant of tested candidate")
    changes = git(root, "diff", "--no-renames", "--name-status", "-z", delta["candidate"], publication).decode().split("\0")
    changes = changes[:-1] if changes[-1] == "" else changes
    require(len(changes) % 2 == 0, "invalid publication diff")
    paths = []
    governance = {".beads/issues.jsonl", "specs/active_frontier.json", "specs/PROGRESS.md"}
    for status, path in zip(changes[::2], changes[1::2]):
        require((status == "M" and path in governance) or
                (status == "A" and path.startswith(("specs/evidence/preferences/", "docs/reviews/preferences/"))),
                "publication changed a proof input or non-additive evidence: " + path)
        paths.append(path)
    require(delta["changed_paths"] == paths, "publication inventory differs from actual Git delta")
    # This post-proof review is bound by the separately selected receipt, not its report.
    review_bundle = Bundle(bundle_root, [delta["review"]])
    review = review_bundle.json(delta["review"])
    closed(review, "schema candidate publication changed_paths reviewer_session disposition", "publication review")
    require(review["schema"] == "datum.preferences.publication-review.v2" and
            review["candidate"] == delta["candidate"] and review["publication"] == publication and
            review["changed_paths"] == paths and review["disposition"] == "approved", "publication review differs")
    text(review["reviewer_session"], "publication reviewer")
    return {**readiness, "state": "production_accepted", "production_accepted": True}
