#!/usr/bin/env python3
"""Shared MCP server test fixtures and fake daemon responses."""

from fake_daemon_support_base import FakeDaemonClientBase
from fake_daemon_support_checks import FakeDaemonClientChecksMixin
from fake_daemon_support_import_map import FakeDaemonClientImportMapMixin
from fake_daemon_support_journal import FakeDaemonClientJournalMixin
from fake_daemon_support_library import FakeDaemonClientLibraryMixin
from fake_daemon_support_mutations import FakeDaemonClientMutationsMixin
from fake_daemon_support_proposals import FakeDaemonClientProposalsMixin
from fake_daemon_support_queries import FakeDaemonClientQueriesMixin
from fake_daemon_support_relationships import FakeDaemonClientRelationshipsMixin
from fake_daemon_support_replacements import FakeDaemonClientReplacementsMixin


class FakeDaemonClient(
    FakeDaemonClientBase,
    FakeDaemonClientChecksMixin,
    FakeDaemonClientImportMapMixin,
    FakeDaemonClientJournalMixin,
    FakeDaemonClientLibraryMixin,
    FakeDaemonClientMutationsMixin,
    FakeDaemonClientProposalsMixin,
    FakeDaemonClientRelationshipsMixin,
    FakeDaemonClientReplacementsMixin,
    FakeDaemonClientQueriesMixin,
):
    pass
