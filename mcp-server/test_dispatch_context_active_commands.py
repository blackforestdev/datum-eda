from __future__ import annotations

import unittest

from server_runtime import StdioToolHost
from test_support import FakeDaemonClient


class TestDispatchContextActiveCommands(unittest.TestCase):
    def test_context_get_preserves_active_context_commands_in_mcp_envelope(self) -> None:
        daemon = FakeDaemonClient()
        host = StdioToolHost(daemon)

        response = host.handle_message(
            {
                "jsonrpc": "2.0",
                "id": 411,
                "method": "tools/call",
                "params": {
                    "name": "datum.context.get",
                    "arguments": {
                        "session": "session-test",
                        "path": "/tmp/context.json",
                        "project_root": "/tmp/native-project",
                    },
                },
            }
        )

        payload = response["result"]["content"][0]["json"]
        commands = payload["result"]["active_context_commands"]
        self.assertEqual(payload["active_context_commands"], commands)
        self.assertEqual(
            commands["artifact_list"],
            "datum-eda artifact list /tmp/native-project",
        )
        self.assertEqual(
            commands["artifact_show"],
            "datum-eda artifact show /tmp/native-project --artifact artifact-gerber",
        )
        self.assertEqual(
            commands["artifact_files"],
            "datum-eda artifact files /tmp/native-project --artifact artifact-gerber",
        )
        self.assertEqual(
            commands["artifact_preview"],
            "datum-eda artifact preview /tmp/native-project --artifact artifact-gerber "
            "--file build/fab/doa2526.gbr",
        )
        self.assertEqual(
            commands["artifact_compare"],
            "datum-eda artifact compare /tmp/native-project --before artifact-previous --after artifact-gerber",
        )
        self.assertEqual(
            commands["artifact_validate"],
            "datum-eda artifact validate /tmp/native-project --artifact artifact-gerber",
        )
        self.assertEqual(
            commands["output_job_generate"],
            "datum-eda artifact generate /tmp/native-project --output-job job-gerber",
        )
        self.assertEqual(
            commands["output_job_start_run"],
            "datum-eda artifact start-output-job-run /tmp/native-project --output-job job-gerber",
        )
        self.assertEqual(
            commands["output_job_cancel_run"],
            "datum-eda artifact cancel-output-job-run /tmp/native-project --run run-gerber-2",
        )
        self.assertEqual(
            commands["proposal_list"],
            "datum-eda proposal list /tmp/native-project",
        )
        self.assertEqual(payload["latest_proposal_id"], "proposal-repair")
        self.assertEqual(payload["visible_proposal_ids"], ["proposal-repair"])
        self.assertEqual(
            commands["proposal_show"],
            "datum-eda proposal show /tmp/native-project --proposal proposal-repair",
        )
        self.assertEqual(
            commands["proposal_preview"],
            "datum-eda proposal preview /tmp/native-project --proposal proposal-repair",
        )
        self.assertEqual(
            commands["proposal_validate"],
            "datum-eda proposal validate /tmp/native-project --proposal proposal-repair",
        )
        self.assertEqual(
            commands["proposal_review_accept"],
            "datum-eda proposal review /tmp/native-project --proposal proposal-repair --status accepted",
        )
        self.assertEqual(
            commands["proposal_review_reject"],
            "datum-eda proposal review /tmp/native-project --proposal proposal-repair --status rejected",
        )
        self.assertEqual(
            commands["proposal_defer"],
            "datum-eda proposal defer /tmp/native-project --proposal proposal-repair",
        )
        self.assertEqual(
            commands["proposal_reject"],
            "datum-eda proposal reject /tmp/native-project --proposal proposal-repair",
        )
        self.assertEqual(
            commands["proposal_accept_apply"],
            "datum-eda proposal accept-apply /tmp/native-project --proposal proposal-repair",
        )
        self.assertEqual(
            commands["proposal_apply"],
            "datum-eda proposal apply /tmp/native-project --proposal proposal-repair",
        )
        self.assertEqual(payload["accepted_transaction_tip"], "transaction-tip")
        self.assertEqual(
            commands["journal_list"],
            "datum-eda journal list /tmp/native-project",
        )
        self.assertEqual(
            commands["journal_show_tip"],
            "datum-eda journal show /tmp/native-project --transaction transaction-tip",
        )
        self.assertEqual(
            commands["journal_undo"],
            "datum-eda journal undo /tmp/native-project",
        )
        self.assertEqual(
            commands["journal_redo"],
            "datum-eda journal redo /tmp/native-project",
        )
        self.assertEqual(
            commands["source_shards"],
            "datum-eda project query /tmp/native-project resolve-debug",
        )
        self.assertEqual(
            commands["check_run"],
            "datum-eda check run /tmp/native-project",
        )
        self.assertEqual(
            commands["check_list"],
            "datum-eda check list /tmp/native-project",
        )
        self.assertEqual(
            commands["check_profiles"],
            "datum-eda check profiles /tmp/native-project",
        )
        self.assertEqual(
            commands["check_fill_zones"],
            "datum-eda check fill-zones /tmp/native-project",
        )
        self.assertEqual(
            commands["check_waive_finding"],
            "datum-eda check waive /tmp/native-project "
            "--fingerprint 'sha256:selected-finding' --rationale '<rationale>'",
        )
        self.assertEqual(
            commands["check_accept_deviation"],
            "datum-eda check accept-deviation /tmp/native-project "
            "--fingerprint 'sha256:selected-finding' --rationale '<rationale>'",
        )
        self.assertIsNone(commands["check_show"])


if __name__ == "__main__":
    unittest.main()
