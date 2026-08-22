#!/usr/bin/env python3
"""EDA MCP Server entrypoint and self-test harness."""

from __future__ import annotations

import argparse
import json
import os
import sys
import unittest

from discovery_scope import load_discovery_scope
from server_runtime import EngineDaemonClient, StdioToolHost, run_server
from streamable_http import create_http_server, load_bearer_token


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
    parser.add_argument("--transport", choices=("stdio", "http"), default="stdio")
    parser.add_argument("--port", type=int)
    parser.add_argument("--token-file")
    parser.add_argument("--allow-origin", action="append", default=[])
    args = parser.parse_args()
    try:
        load_discovery_scope(args.discovery)
        if args.transport == "stdio":
            if args.port is not None or args.token_file or args.allow_origin:
                parser.error("HTTP options require --transport http")
            run_server()
        else:
            if args.port is None or args.token_file is None:
                parser.error("HTTP transport requires --port and --token-file")
            token = load_bearer_token(args.token_file)
            server = create_http_server(
                args.port,
                token,
                args.allow_origin,
                StdioToolHost(EngineDaemonClient()),
            )
            print(
                json.dumps(
                    {
                        "level": "info",
                        "component": "datum-mcp",
                        "transport": "streamable-http",
                        "endpoint": f"http://127.0.0.1:{server.server_port}/mcp",
                    }
                ),
                file=sys.stderr,
                flush=True,
            )
            server.serve_forever()
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
