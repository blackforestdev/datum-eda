#!/usr/bin/env bash
set -euo pipefail

python3 scripts/check_progress_coverage.py
python3 scripts/check_alignment.py
python3 scripts/check_alignment.py --run-gates
python3 scripts/check_file_size_budgets.py
python3 scripts/check_decomposition_coverage.py
python3 scripts/check_test_file_sizes.py --max-lines 700
python3 mcp-server/server.py --self-test
