#!/usr/bin/env python3
"""Claim-state reporting for the roadmap selector.

Extracted from test_project_status.py under decision 022: these tests own one
cohesive behaviour - how a frontier claim's real condition is reported - and the
parent file exceeded its 700-line ceiling when they were added.
"""

from __future__ import annotations
import unittest
from datetime import datetime, timezone

from project_task_details import resolve_claim_state


class ClaimStateReporting(unittest.TestCase):
    """A claim's real condition must never be collapsed into active-or-none.

    Regression for the selector reporting `live claim: none` while a valid claim
    sat in active_frontier.json, and reporting `active` for a lapsed lease.
    """

    NOW = datetime(2026, 8, 29, 20, 0, 0, tzinfo=timezone.utc)
    CLAIM = {
        "agent": "codex",
        "claimed_at": "2026-08-29T19:00:45Z",
        "heartbeat_at": "2026-08-29T19:00:45Z",
        "expires_at": "2026-08-30T03:00:45Z",
    }

    def test_absent_claim_reports_none(self):
        state, text = resolve_claim_state({"state": "ready"}, 8, self.NOW)
        self.assertEqual("none", state)
        self.assertEqual("none", text)

    def test_live_claim_names_agent_and_expiry(self):
        item = {"state": "in_progress", "claim": self.CLAIM}
        state, text = resolve_claim_state(item, 8, self.NOW)
        self.assertEqual("active", state)
        self.assertIn("codex", text)
        self.assertIn("2026-08-30T03:00:45Z", text)

    def test_lapsed_lease_is_expired_not_active(self):
        claim = dict(self.CLAIM, expires_at="2026-08-29T19:30:00Z")
        state, text = resolve_claim_state(
            {"state": "in_progress", "claim": claim}, 8, self.NOW
        )
        self.assertEqual("expired", state)
        self.assertIn("EXPIRED", text)
        self.assertNotEqual("none", text)

    def test_claim_outside_in_progress_is_orphaned_not_none(self):
        state, text = resolve_claim_state(
            {"state": "ready", "claim": self.CLAIM}, 8, self.NOW
        )
        self.assertEqual("orphaned", state)
        self.assertIn("ORPHANED", text)
        self.assertNotEqual("none", text)

    def test_missing_expiry_falls_back_to_ttl(self):
        claim = {k: v for k, v in self.CLAIM.items() if k != "expires_at"}
        state, _ = resolve_claim_state(
            {"state": "in_progress", "claim": claim}, 8, self.NOW
        )
        self.assertEqual("active", state)
        lapsed = dict(claim, heartbeat_at="2026-08-29T18:00:00Z")
        stale, _ = resolve_claim_state(
            {"state": "in_progress", "claim": lapsed}, 1, self.NOW
        )
        self.assertEqual("expired", stale)


if __name__ == "__main__":
    unittest.main()
