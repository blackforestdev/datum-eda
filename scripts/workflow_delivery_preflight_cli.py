#!/usr/bin/env python3
"""Inspect exact publication preconditions and retain a Git-common JSON log.

Requires -I -S -B and an explicit inspection or owner activation mode.
A caller-pinned request digest identifies input bytes; it is not owner approval.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys


def authenticate_activation_runtime(args):
    """Authenticate bootstrap before importing any adjacent implementation."""
    raw = args.request.read_bytes()
    if hashlib.sha256(raw).hexdigest() != args.request_sha256:
        raise ValueError("request bytes differ from explicit SHA-256")
    authority = json.loads(raw)["authority"]
    if not isinstance(authority, str) or not re.fullmatch(r"[a-f0-9]{40}|[a-f0-9]{64}", authority):
        raise ValueError("full activation authority required")
    env = {"PATH": os.defpath, "LC_ALL": "C", "GIT_CONFIG_NOSYSTEM": "1",
           "GIT_CONFIG_GLOBAL": os.devnull, "GIT_GRAFT_FILE": os.devnull}
    def read_git(*arguments):
        result = subprocess.run(["git", "--no-replace-objects", "--no-optional-locks", *arguments],
            cwd=args.root, env=env, capture_output=True, check=False)
        if result.returncode:
            raise ValueError("activation runtime authentication failed: " + result.stderr.decode("utf-8", "replace"))
        return result.stdout
    common = Path(read_git("rev-parse", "--path-format=absolute", "--git-common-dir").decode().strip()).resolve(strict=True)
    directory = common / "datum-wdq/trusted" / authority / "scripts"
    if Path(__file__).absolute() != directory / "workflow_delivery_preflight_cli.py" or directory.resolve() != directory:
        raise ValueError("activation must execute the exact prepared Git-common runtime")
    for name in ("workflow_delivery_preflight_cli.py", "workflow_delivery_bootstrap.py"):
        path = directory / name
        source_bytes = path.read_bytes()
        if path.is_symlink() or source_bytes != read_git("show", authority + ":scripts/" + name):
            raise ValueError("activation bootstrap bytes differ from selected authority")
    # Execute authenticated source while sys.path is still isolated. Importing
    # from the bundle first could consume an unverified cached bootstrap .pyc.
    namespace = {"__name__": "_datum_activation_bootstrap", "__file__": str(path)}
    exec(compile(source_bytes, str(path), "exec"), namespace)
    namespace["verify_support"](args.root, authority)
    sys.path.insert(0, str(directory))


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--inspect", action="store_true")
    mode.add_argument("--inspect-state", action="store_true")
    mode.add_argument("--inspect-promotion", action="store_true")
    mode.add_argument("--inspect-review", action="store_true")
    mode.add_argument("--activate", action="store_true")
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--request", type=Path, required=True)
    parser.add_argument("--request-sha256", required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--owner-response", type=Path)
    parser.add_argument("--owner-response-sha256")
    args = parser.parse_args(argv)
    report = {"kind": "datum.workflow-delivery.preflight-diagnostic", "ok": False,
              "activation_asserted": False, "publication_authorized": False, "findings": []}
    if args.inspect_review:
        report["evidence_validated"] = False
    output = None
    try:
        if not (sys.flags.isolated and sys.flags.no_site and sys.flags.dont_write_bytecode):
            raise ValueError("preflight inspection requires Python -I -S -B")
        if args.activate:
            authenticate_activation_runtime(args)
        # Activation authenticates the complete bundle first. Inspection remains
        # non-authoritative, but also must not consume adjacent bytecode caches.
        source_bootstrap = Path(__file__).with_name("workflow_delivery_source_only.py")
        source_namespace = {"__name__": "_datum_source_bootstrap"}
        exec(compile(source_bootstrap.read_bytes(), str(source_bootstrap), "exec"), source_namespace)
        source_namespace["install"](source_bootstrap.parent)
        # Import only this explicitly executed preparation source, not cwd or
        # PYTHONPATH. This command never promotes its own runtime as authority.
        sys.path.insert(0, str(Path(__file__).resolve().parent))
        from workflow_delivery_activation_preflight import preflight, inspect_promotion_candidate
        from workflow_delivery_activation_state import inspect_activation_state
        from workflow_delivery_io import canonical_json, parse_json, sha256
        from workflow_delivery_shapes import closed
        from workflow_delivery_support_store import support_locations, proposed_local_trust
        from workflow_delivery_support_bundle import verify_bundle
        from workflow_delivery_environments import load_environments
        from workflow_delivery_tree import Tree
        from workflow_delivery_trust import POLICY_PATH, policy_shape
        from workflow_delivery_publication_delta import inspect_activation_response
        from workflow_delivery_workspace_authority import promotion_workspace

        if bool(args.owner_response) != bool(args.owner_response_sha256):
            raise ValueError("owner response path and externally selected digest are required together")
        if args.inspect_state and args.owner_response:
            raise ValueError("state observation does not inspect owner responses")
        if args.inspect_review and args.owner_response:
            raise ValueError("review inspection does not consume owner activation responses")
        if (args.inspect_promotion or args.activate) and not args.owner_response:
            raise ValueError("promotion inspection requires separately selected exact owner response")

        raw = args.request.read_bytes()
        if not re.fullmatch(r"[0-9a-f]{64}", args.request_sha256) or sha256(raw) != args.request_sha256:
            raise ValueError("request bytes differ from explicit SHA-256")
        request = parse_json(raw, str(args.request))
        fields = "schema_version kind base candidate authority input_roots prior_local_trust"
        expected_kind = "datum.workflow-delivery.preflight-request"
        if args.inspect_state:
            fields += " proposed_local_trust"
            expected_kind = "datum.workflow-delivery.activation-state-request"
        else:
            fields += " publication_review proposed_local_trust environment_path"
            if args.inspect_promotion:
                expected_kind = "datum.workflow-delivery.promotion-inspection-request"
            if args.inspect_review:
                expected_kind = "datum.workflow-delivery.review-inspection-request"
            if args.activate:
                fields += " coordination"
                expected_kind = "datum.workflow-delivery.activation-request"
        closed(request, fields, "preflight request")
        if (type(request["schema_version"]) is not int or request["schema_version"] != 1
                or request["kind"] != expected_kind):
            raise ValueError("closed preflight request version 1 required")
        if (type(request["input_roots"]) is not list
                or not all(type(path) is str for path in request["input_roots"])):
            raise ValueError("explicit string input roots required")
        locations = support_locations(args.root, request["authority"])
        output = args.output
        if (not output.is_absolute() or output.parent != locations["logs"]
                or output.resolve() != output
                or not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]*\.json", output.name)):
            raise ValueError("output must be a literal JSON filename in the resolved Git-common WDQ logs directory")
        locations["logs"].mkdir(parents=True, exist_ok=True)
        # Reserve one explicit log. Never overwrite an earlier run, including
        # interrupted or failed runs. An interrupted write is not a success log.
        with output.open("xb") as stream:
            payload = {"schema_version": 1, "request_sha256": args.request_sha256,
                       "request": request, "activation_asserted": False,
                       "publication_authorized": False}
            if args.inspect_review:
                payload["evidence_validated"] = False
            try:
                if args.inspect_state:
                    payload["activation_state"] = inspect_activation_state(args.root,
                        base=request["base"], candidate=request["candidate"], input_roots=request["input_roots"],
                        prior_trust=request["prior_local_trust"], proposed_trust=request["proposed_local_trust"])
                else:
                    if args.owner_response:
                        response = args.owner_response
                        response_directory = locations["store"] / "owner-responses"
                        if (not response.is_absolute() or response.parent != response_directory
                                or response.resolve() != response or not response.is_file()
                                or response.is_symlink() or response.suffix != ".json"):
                            raise ValueError("owner response must be a separate nonredirected JSON file in Git-common owner-responses")
                        payload["owner_response"] = inspect_activation_response(response.read_bytes(),
                            response_sha256=args.owner_response_sha256, request_sha256=args.request_sha256)
                    proposed = proposed_local_trust(args.root, authority=request["authority"],
                        base=request["base"], environment_path=request["environment_path"])
                    if canonical_json(request["proposed_local_trust"]) != canonical_json(proposed):
                        raise ValueError("proposed local trust differs from the exact requested pins and support locations")
                    support = verify_bundle(args.root, request["authority"])
                    authority_tree = Tree(args.root, revision=request["authority"])
                    candidate_tree = Tree(args.root, revision=request["candidate"])
                    environment_path = request["environment_path"]
                    if candidate_tree.read(environment_path, committed=True) != authority_tree.read(environment_path, committed=True):
                        raise ValueError("proposed environment differs between candidate and selected authority")
                    environments = load_environments(candidate_tree, environment_path,
                        policy_shape(authority_tree.json(POLICY_PATH)), authority=authority_tree)
                    payload["environment_selection"] = {"path": environment_path,
                        "sha256": sha256(authority_tree.read(environment_path, committed=True)),
                        "enrolled_keys": sorted(environments)}
                    workspace = promotion_workspace(args.root, candidate=request["candidate"],
                                                    authority=request["authority"])
                    payload["preflight"] = preflight(args.root, base=request["base"],
                        candidate=request["candidate"], input_roots=request["input_roots"],
                        expected_local_trust=request["prior_local_trust"],
                        publication_review=request["publication_review"], workspace=workspace)
                    if args.inspect_promotion or args.inspect_review:
                        inspector = inspect_promotion_candidate
                        inspection_key = "promotion_inspection"
                        if args.inspect_review:
                            from workflow_delivery_review_inspection import inspect_review_candidate
                            inspector = inspect_review_candidate
                            inspection_key = "review_inspection"
                        payload[inspection_key] = inspector(args.root,
                            base=request["base"], candidate=request["candidate"], authority=request["authority"],
                            environment_path=environment_path, publication_review=request["publication_review"])
                        repeated = preflight(args.root, base=request["base"],
                            candidate=request["candidate"], input_roots=request["input_roots"],
                            expected_local_trust=request["prior_local_trust"],
                            publication_review=request["publication_review"], workspace=workspace)
                        if canonical_json(repeated) != canonical_json(payload["preflight"]):
                            raise ValueError("publication state changed during delivery evidence inspection")
                    if verify_bundle(args.root, request["authority"]) != support:
                        raise ValueError("proposed support changed during inspection")
                    payload["support_bundle"] = support
                    if args.activate:
                        from workflow_delivery_activation import activate
                        progress = output.with_suffix(".events.jsonl")
                        payload["progress_log"] = str(progress)
                        with progress.open("xb") as events:
                            directory_fd = os.open(output.parent, os.O_RDONLY | os.O_DIRECTORY)
                            try:
                                os.fsync(directory_fd)
                            finally:
                                os.close(directory_fd)
                            def record(event):
                                event["request_sha256"] = args.request_sha256
                                events.write(canonical_json(event))
                                events.flush()
                                os.fsync(events.fileno())
                            record({"stage": "requested", "response_sha256": args.owner_response_sha256})
                            payload["activation"] = activate(args.root, request, record=record)
                        payload["activation_asserted"] = True
                payload["ok"] = True
            except (ValueError, OSError, RuntimeError, subprocess.TimeoutExpired, KeyboardInterrupt) as error:
                payload.update(ok=False, error={"type": type(error).__name__, "detail": str(error)})
                if args.activate:
                    try:
                        payload["observed_activation_state"] = inspect_activation_state(args.root,
                            base=request["base"], candidate=request["candidate"], input_roots=request["input_roots"],
                            prior_trust=request["prior_local_trust"], proposed_trust=request["proposed_local_trust"],
                            workspace=locals().get("workspace"))
                    except (ValueError, OSError, RuntimeError) as observation_error:
                        payload["state_observation_error"] = str(observation_error)
            encoded = canonical_json(payload)
            stream.write(encoded)
            stream.flush()
            os.fsync(stream.fileno())
        report.update(ok=payload["ok"], log=str(output), log_sha256=hashlib.sha256(encoded).hexdigest(),
                      request_sha256=args.request_sha256)
        report["activation_asserted"] = payload["activation_asserted"]
        if "observed_activation_state" in payload:
            report["state"] = payload["observed_activation_state"]["state"]
        if "activation_state" in payload:
            report["state"] = payload["activation_state"]["state"]
            report["ok"] = report["state"] == "not_started"
            report["observation_completed"] = True
        if not payload["ok"]:
            report["findings"].append(payload["error"])
    except (ValueError, OSError, RuntimeError, ImportError, KeyError, TypeError) as error:
        report["findings"].append({"type": type(error).__name__, "detail": str(error)})
    print(json.dumps(report, sort_keys=True), flush=True)
    return 0 if report["ok"] else 2


if __name__ == "__main__":
    raise SystemExit(main())
