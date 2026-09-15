#!/usr/bin/env python3
"""Compatibility entry point for decision-022 source-health governance.

The former 1,500-line flag ledger was superseded by
``scripts/check_source_health.py``. Keep this name temporarily for external
automation while routing every invocation through the blocking policy engine.
"""

from check_source_health import main


if __name__ == "__main__":
    raise SystemExit(main())
