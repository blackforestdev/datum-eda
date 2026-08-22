#!/usr/bin/env python3
"""EDA MCP Server entrypoint and self-test harness."""

from __future__ import annotations

import argparse
import json
import os
import sys
import unittest

from discovery_scope import load_discovery_scope
from server_runtime import run_server


def run_self_tests() -> int:
    suite = unittest.defaultTestLoader.discover(
        start_dir=os.path.dirname(__file__),
        pattern="test_*.py",
        top_level_dir=os.path.dirname(__file__),
    )
    result = unittest.TextTestRunner(verbosity=1).run(suite)
    return 0 if result.wasSuccessful() else 1


if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "--self-test":
        sys.exit(run_self_tests())
    parser = argparse.ArgumentParser(description="Datum MCP stdio broker")
    parser.add_argument("--discovery", required=True)
    args = parser.parse_args()
    try:
        load_discovery_scope(args.discovery)
        run_server()
    except Exception as exc:
        print(
            json.dumps(
                {
                    "level": "error",
                    "component": "datum-mcp",
                    "message": str(exc)[:1024],
                }
            ),
            file=sys.stderr,
            flush=True,
        )
        sys.exit(2)
