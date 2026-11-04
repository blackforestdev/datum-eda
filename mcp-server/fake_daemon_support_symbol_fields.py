"""Shared field caller response for MCP dispatch tests."""
from server_runtime import JsonRpcResponse


class FakeDaemonSymbolFieldsMixin:
    def set_symbol_field(self, path: str, symbol: str, field: str, expected_model_revision: str, expected_tip_transaction: str, value: str | None = None, clear: bool = False, instance_path: list[str] | None = None) -> JsonRpcResponse:
        self.calls.append(("set_symbol_field", path, symbol, field, expected_model_revision, expected_tip_transaction, value, clear, instance_path))
        return self._component_modified_response(1319, symbol, f"set_symbol_field {symbol}")
