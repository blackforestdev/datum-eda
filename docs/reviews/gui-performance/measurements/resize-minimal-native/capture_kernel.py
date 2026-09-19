#!/usr/bin/env python3
"""Disabled diagnostic: the prior kernel tracing setup caused excessive CPU load."""
import sys

if __name__ == '__main__':
    sys.exit('Capture disabled after excessive CPU during kernel tracer setup. Do not rerun. The prior source is retained as capture_kernel_failed_attempt.py.txt for analysis. No tracing controls were touched by this invocation.')
