#!/usr/bin/env python3
"""Fake daemon client journal responses for MCP tests."""

from __future__ import annotations

from server_runtime import JsonRpcResponse


def _journal_transaction(transaction: str = "txn-test", status: str = "applied") -> dict:
    return {
        "transaction_id": transaction,
        "id": transaction,
        "project_id": "project-test",
        "model_revision": "model-rev-test",
        "status": status,
        "operation_count": 2,
        "operations": [
            {
                "operation_id": "op-bind-ci",
                "action": "bind_component_instance",
                "target": {"kind": "component_instance", "id": "ci-test"},
                "status": "applied",
            },
            {
                "operation_id": "op-review-proposal",
                "action": "review_proposal",
                "target": {"kind": "proposal", "id": "proposal-test"},
                "status": "applied",
            },
        ],
    }


class FakeDaemonClientJournalMixin:
    def get_journal_list(self, path: str) -> JsonRpcResponse:
        self.calls.append(("get_journal_list", path))
        transaction = _journal_transaction()
        return JsonRpcResponse(
            "2.0",
            132,
            {
                "contract": "project_transaction_journal_list_v1",
                "project_root": path,
                "project_id": "project-test",
                "model_revision": "model-rev-test",
                "count": 1,
                "transaction_count": 1,
                "cursor_index": 1,
                "can_undo": True,
                "can_redo": False,
                "transactions": [transaction],
            },
            None,
        )

    def get_journal_transaction(self, path: str, transaction: str) -> JsonRpcResponse:
        self.calls.append(("get_journal_transaction", path, transaction))
        return JsonRpcResponse(
            "2.0",
            133,
            {
                "contract": "project_transaction_journal_record_v1",
                "project_root": path,
                "project_id": "project-test",
                "model_revision": "model-rev-test",
                "transaction_id": transaction,
                "transaction": _journal_transaction(transaction),
            },
            None,
        )

    def journal_undo(
        self,
        path: str,
        expected_model_revision: str | None = None,
        expected_tip_transaction: str | None = None,
    ) -> JsonRpcResponse:
        self.calls.append(
            ("journal_undo", path, expected_model_revision, expected_tip_transaction)
        )
        return JsonRpcResponse(
            "2.0",
            134,
            {
                "contract": "project_transaction_journal_mutation_v1",
                "project_id": "project-test",
                "model_revision": "model-rev-test",
                "action": "undo",
                "status": "applied",
                "guard": {
                    "checked": expected_model_revision is not None or expected_tip_transaction is not None,
                    "current_model_revision": "model-rev-test",
                    "expected_model_revision": expected_model_revision,
                    "current_tip_transaction": "txn-tip-current",
                    "expected_tip_transaction": expected_tip_transaction,
                },
                "transaction_id": "txn-test",
                "cursor_index": 0,
                "can_undo": False,
                "can_redo": True,
            },
            None,
        )

    def journal_redo(
        self,
        path: str,
        expected_model_revision: str | None = None,
        expected_tip_transaction: str | None = None,
    ) -> JsonRpcResponse:
        self.calls.append(
            ("journal_redo", path, expected_model_revision, expected_tip_transaction)
        )
        return JsonRpcResponse(
            "2.0",
            135,
            {
                "contract": "project_transaction_journal_mutation_v1",
                "project_id": "project-test",
                "model_revision": "model-rev-test",
                "action": "redo",
                "status": "applied",
                "guard": {
                    "checked": expected_model_revision is not None or expected_tip_transaction is not None,
                    "current_model_revision": "model-rev-test",
                    "expected_model_revision": expected_model_revision,
                    "current_tip_transaction": "txn-tip-current",
                    "expected_tip_transaction": expected_tip_transaction,
                },
                "transaction_id": "txn-test",
                "cursor_index": 1,
                "can_undo": True,
                "can_redo": False,
            },
            None,
        )
