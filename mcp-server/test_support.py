#!/usr/bin/env python3
"""Shared MCP server test fixtures and fake daemon responses."""

from fake_daemon_support_base import FakeDaemonClientBase
from fake_daemon_support_mutations import FakeDaemonClientMutationsMixin
from fake_daemon_support_queries import FakeDaemonClientQueriesMixin
from fake_daemon_support_replacements import FakeDaemonClientReplacementsMixin


class FakeDaemonClient(
    FakeDaemonClientBase,
    FakeDaemonClientMutationsMixin,
    FakeDaemonClientReplacementsMixin,
    FakeDaemonClientQueriesMixin,
):
    pass
