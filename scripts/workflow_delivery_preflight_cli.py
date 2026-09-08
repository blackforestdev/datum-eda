#!/usr/bin/env python3
"""Inspect exact publication preconditions and retain a Git-common JSON log.

Requires -I -S -B and --inspect. There is deliberately no activation command.
A caller-pinned request digest identifies input bytes; it is not owner approval.
"""

import argparse
import hashlib
import json
from pathlib import Path
import re
import sys


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--inspect", action="store_true", required=True)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--request", type=Path, required=True)
    parser.add_argument("--request-sha256", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args(argv)
    report = {"kind": "datum.workflow-delivery.preflight-diagnostic", "ok": False,
              "activation_asserted": False, "publication_authorized": False, "findings": []}
    output = None
    try:
        if not (sys.flags.isolated and sys.flags.no_site and sys.flags.dont_write_bytecode):
            raise ValueError("preflight inspection requires Python -I -S -B")
        # Import only this explicitly executed preparation source, not cwd or
        # PYTHONPATH. This command never promotes its own runtime as authority.
        sys.path.insert(0, str(Path(__file__).resolve().parent))
        from workflow_delivery_activation_preflight import preflight
        from workflow_delivery_io import canonical_json, parse_json, sha256
        from workflow_delivery_shapes import closed
        from workflow_delivery_support_store import support_locations
        from workflow_delivery_support_bundle import verify_bundle

        raw = args.request.read_bytes()
        if not re.fullmatch(r"[0-9a-f]{64}", args.request_sha256) or sha256(raw) != args.request_sha256:
            raise ValueError("request bytes differ from explicit SHA-256")
        request = parse_json(raw, str(args.request))
        closed(request, "schema_version kind base candidate authority input_roots prior_local_trust", "preflight request")
        if (type(request["schema_version"]) is not int or request["schema_version"] != 1
                or request["kind"] != "datum.workflow-delivery.preflight-request"):
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
            try:
                support = verify_bundle(args.root, request["authority"])
                payload["preflight"] = preflight(args.root, base=request["base"],
                    candidate=request["candidate"], input_roots=request["input_roots"],
                    expected_local_trust=request["prior_local_trust"])
                if verify_bundle(args.root, request["authority"]) != support:
                    raise ValueError("proposed support changed during inspection")
                payload["support_bundle"] = support
                payload["ok"] = True
            except (ValueError, OSError, RuntimeError) as error:
                payload.update(ok=False, error={"type": type(error).__name__, "detail": str(error)})
            encoded = canonical_json(payload)
            stream.write(encoded)
            stream.flush()
        report.update(ok=payload["ok"], log=str(output), log_sha256=hashlib.sha256(encoded).hexdigest(),
                      request_sha256=args.request_sha256)
        if not payload["ok"]:
            report["findings"].append(payload["error"])
    except (ValueError, OSError, RuntimeError, ImportError) as error:
        report["findings"].append({"type": type(error).__name__, "detail": str(error)})
    print(json.dumps(report, sort_keys=True), flush=True)
    return 0 if report["ok"] else 2


if __name__ == "__main__":
    raise SystemExit(main())
