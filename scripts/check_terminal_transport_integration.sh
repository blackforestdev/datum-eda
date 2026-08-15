#!/usr/bin/env bash
set -euo pipefail

platform="$(uname -s)"
if [[ "$platform" != "Linux" ]]; then
  echo "Terminal transport integration skipped on $platform (Datum production target: Linux)."
  exit 0
fi

if [[ ! -c /dev/ptmx ]]; then
  echo "Terminal transport integration failed: /dev/ptmx is not a character device." >&2
  exit 1
fi

echo "== terminal transport: process semantics on Linux PTY =="
cargo test -p datum-gui-app terminal_process_semantics_tests -- --nocapture

echo "== terminal transport: concurrent session isolation on Linux PTY =="
cargo test -p datum-gui-app terminal_session_isolation_tests -- --nocapture
