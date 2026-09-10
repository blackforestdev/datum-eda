#!/usr/bin/env python3
"""Capture one implemented infrastructure case from an exact retained recipe.

Use Python -I -S -B. Output is raw case evidence, not an assembled delivery
packet, complete matrix, reviewed input closure, readiness or owner acceptance.
No activation or live development mutation is available through this command.
Destinations are fresh direct children of the resolved Git-common WDQ proposal
store; arbitrary worktree or Documents output locations are not permitted.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--capture", action="store_true", required=True)
    parser.add_argument("--packet", type=Path, required=True)
    parser.add_argument("--recipe-sha256", required=True)
    parser.add_argument("--destination", type=Path, required=True)
    parser.add_argument("--case", required=True)
    parser.add_argument("--surface", choices=("C", "R", "S-check", "S-details", "H"), required=True)
    args = parser.parse_args(argv)
    report = {"kind": "datum.workflow-delivery.case-capture", "ok": False,
              "complete_matrix": False, "input_closure_verified": False,
              "readiness_asserted": False, "acceptance_asserted": False, "findings": []}
    destination = None
    try:
        if not (sys.flags.isolated and sys.flags.no_site and sys.flags.dont_write_bytecode):
            raise ValueError("capture command requires Python -I -S -B")
        runtime = Path(__file__).resolve().parents[1]
        # The owner explicitly executes this preparation source. Import neither
        # cwd nor PYTHONPATH; this is not a trust-installation bootstrap.
        sys.path.insert(0, str(runtime / "scripts"))
        from workflow_delivery_capture_cases import run_case
        from workflow_delivery_capture_hook import run_hook_case
        from workflow_delivery_capture_command import save
        from workflow_delivery_capture_state import git
        from workflow_delivery_io import sha256
        from workflow_delivery_support_store import support_locations

        packet = args.packet.resolve(strict=True)
        recipe_path = packet / ("result.json" if args.case == "INFRA-S04-11" else "recipe.json")
        recipe = recipe_path.read_bytes()
        if not re.fullmatch(r"[a-f0-9]{64}", args.recipe_sha256) or sha256(recipe) != args.recipe_sha256:
            raise ValueError("retained recipe differs from explicit SHA-256")
        target = args.destination
        observed_head = git(runtime, "rev-parse", "HEAD").decode().strip()
        locations = support_locations(runtime, observed_head)
        if (not target.is_absolute() or target.resolve() != target or target.exists()
                or target.is_symlink() or target.parent != locations["proposals"]
                or target.is_relative_to(packet)
                or packet.is_relative_to(target)):
            raise ValueError("fresh canonical destination directly under Git-common WDQ proposals and outside packet required")
        locations["proposals"].mkdir(parents=True, exist_ok=True)
        target.mkdir(exist_ok=False)
        destination = target
        home = target / "home"
        home.mkdir()
        environment = {"PATH": os.defpath, "HOME": str(home), "XDG_CONFIG_HOME": str(home),
                       "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": os.devnull, "LANG": "C"}
        driver = {"recipe_sha256": args.recipe_sha256, "case_id": args.case, "surface": args.surface,
                  "runtime_root": str(runtime), "driver_sha256": sha256(Path(__file__).read_bytes()),
                  "observed_source_head": observed_head,
                  "source_pin_verified": False, "acceptance_asserted": False}
        save(target / "driver-input.json", driver)
        if args.case == "INFRA-S04-11":
            from workflow_delivery_roadmap_snapshot import capture_roadmap_snapshot
            result = capture_roadmap_snapshot(packet, target / "case", runtime, environment, surface=args.surface)
        elif args.surface == "H":
            result = run_hook_case(packet, target / "case", environment, case_id=args.case)
        else:
            result = run_case(packet, target / "case", runtime, environment,
                              case_id=args.case, surface=args.surface)
        if recipe_path.read_bytes() != recipe:
            raise ValueError("retained recipe changed during capture")
        report.update(ok=True, assessment=result, recipe_sha256=args.recipe_sha256)
    except (ValueError, OSError, RuntimeError, ImportError, KeyError, subprocess.SubprocessError) as error:
        report["findings"].append({"type": type(error).__name__, "detail": str(error)})
    if destination is not None:
        report["destination"] = str(destination)
        encoded = (json.dumps(report, sort_keys=True) + "\n").encode()
        with (destination / "driver-result.json").open("xb") as stream:
            stream.write(encoded)
        report["driver_result_sha256"] = hashlib.sha256(encoded).hexdigest()
    print(json.dumps(report, sort_keys=True), flush=True)
    return 0 if report["ok"] else 2


if __name__ == "__main__":
    raise SystemExit(main())
