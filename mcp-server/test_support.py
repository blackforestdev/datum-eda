#!/usr/bin/env python3
"""Shared MCP server test fixtures and fake daemon responses."""

import json

from fake_daemon_support_base import FakeDaemonClientBase
from fake_daemon_support_checks import FakeDaemonClientChecksMixin
from fake_daemon_support_import_map import FakeDaemonClientImportMapMixin
from fake_daemon_support_journal import FakeDaemonClientJournalMixin
from fake_daemon_support_library import FakeDaemonClientLibraryMixin
from fake_daemon_support_mutations import FakeDaemonClientMutationsMixin
from fake_daemon_support_proposals import FakeDaemonClientProposalsMixin
from fake_daemon_support_preferences import FakeDaemonClientPreferencesMixin
from fake_daemon_support_queries import FakeDaemonClientQueriesMixin
from fake_daemon_support_relationships import FakeDaemonClientRelationshipsMixin
from fake_daemon_support_replacements import FakeDaemonClientReplacementsMixin
from fake_daemon_support_units import FakeDaemonClientUnitsMixin
from fake_daemon_support_symbol_fields import FakeDaemonSymbolFieldsMixin


class FakeDaemonClient(
    FakeDaemonSymbolFieldsMixin,
    FakeDaemonClientBase,
    FakeDaemonClientChecksMixin,
    FakeDaemonClientImportMapMixin,
    FakeDaemonClientJournalMixin,
    FakeDaemonClientLibraryMixin,
    FakeDaemonClientMutationsMixin,
    FakeDaemonClientProposalsMixin,
    FakeDaemonClientPreferencesMixin,
    FakeDaemonClientRelationshipsMixin,
    FakeDaemonClientReplacementsMixin,
    FakeDaemonClientQueriesMixin,
    FakeDaemonClientUnitsMixin,
):
    pass


def decode_tool_payload(response):
    """Decode the standard MCP text block while keeping Datum payload assertions."""
    block = response["result"]["content"][0]
    assert block["type"] == "text", block
    return json.loads(block["text"])
