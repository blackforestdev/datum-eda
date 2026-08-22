#!/usr/bin/env python3
"""Authenticated loopback-only MCP Streamable HTTP transport."""

from __future__ import annotations

import hmac
from http import HTTPStatus
from http.server import BaseHTTPRequestHandler, HTTPServer
import json
import os
from pathlib import Path
import stat
from typing import Any

from stdio_tool_host import StdioToolHost


MAX_REQUEST_BYTES = 16 * 1024 * 1024
MAX_TOKEN_BYTES = 4096
SUPPORTED_PROTOCOL_VERSIONS = {"2025-03-26", "2025-06-18"}


def load_bearer_token(path: str | os.PathLike[str]) -> str:
    token_path = Path(path).resolve(strict=True)
    metadata = token_path.stat()
    if not stat.S_ISREG(metadata.st_mode):
        raise ValueError("MCP token path must be a regular file")
    if stat.S_IMODE(metadata.st_mode) & 0o077:
        raise ValueError("MCP token file must not grant group or other permissions")
    payload = token_path.read_bytes()
    if not payload or len(payload) > MAX_TOKEN_BYTES:
        raise ValueError("MCP token file must contain 1..4096 bytes")
    try:
        token = payload.decode("ascii").strip()
    except UnicodeDecodeError as exc:
        raise ValueError("MCP token must be ASCII") from exc
    if not token or any(character.isspace() for character in token):
        raise ValueError("MCP token must be one non-whitespace ASCII value")
    return token


def create_http_server(
    port: int,
    bearer_token: str,
    allowed_origins: list[str],
    host: StdioToolHost,
) -> HTTPServer:
    if not allowed_origins or any(not _valid_origin(origin) for origin in allowed_origins):
        raise ValueError("at least one exact http(s) browser Origin is required")
    handler = _handler_type(host, bearer_token, frozenset(allowed_origins))
    return HTTPServer(("127.0.0.1", port), handler)


def _handler_type(
    host: StdioToolHost, bearer_token: str, allowed_origins: frozenset[str]
) -> type[BaseHTTPRequestHandler]:
    class DatumMcpHttpHandler(BaseHTTPRequestHandler):
        server_version = "DatumMCP/0.1"
        sys_version = ""

        def do_POST(self) -> None:
            if self.path != "/mcp":
                self._empty(HTTPStatus.NOT_FOUND)
                return
            if not self._authorized():
                return
            if not self._valid_protocol_headers():
                return
            length = self._content_length()
            if length is None:
                return
            try:
                message = json.loads(self.rfile.read(length))
            except (UnicodeError, json.JSONDecodeError):
                self._json(HTTPStatus.BAD_REQUEST, _error(-32700, "parse error"))
                return
            if not isinstance(message, dict):
                self._json(HTTPStatus.BAD_REQUEST, _error(-32600, "invalid request"))
                return
            response = host.handle_message(message)
            if response is None:
                self._empty(HTTPStatus.ACCEPTED)
            else:
                self._json(HTTPStatus.OK, response)

        def do_GET(self) -> None:
            self._empty(HTTPStatus.METHOD_NOT_ALLOWED, allow="POST")

        def do_DELETE(self) -> None:
            self._empty(HTTPStatus.METHOD_NOT_ALLOWED, allow="POST")

        def log_message(self, _format: str, *_args: Any) -> None:
            return

        def _authorized(self) -> bool:
            origin = self.headers.get("Origin")
            if origin is not None and origin not in allowed_origins:
                self._empty(HTTPStatus.FORBIDDEN)
                return False
            authorization = self.headers.get("Authorization", "")
            expected = f"Bearer {bearer_token}"
            if not hmac.compare_digest(authorization, expected):
                self._empty(HTTPStatus.UNAUTHORIZED)
                return False
            return True

        def _valid_protocol_headers(self) -> bool:
            content_type = self.headers.get("Content-Type", "").split(";", 1)[0].strip()
            if content_type != "application/json":
                self._empty(HTTPStatus.UNSUPPORTED_MEDIA_TYPE)
                return False
            accepted = {
                item.split(";", 1)[0].strip()
                for item in self.headers.get("Accept", "").split(",")
            }
            if not {"application/json", "text/event-stream"}.issubset(accepted):
                self._empty(HTTPStatus.NOT_ACCEPTABLE)
                return False
            protocol = self.headers.get("MCP-Protocol-Version", "2025-03-26")
            if protocol not in SUPPORTED_PROTOCOL_VERSIONS:
                self._empty(HTTPStatus.BAD_REQUEST)
                return False
            return True

        def _content_length(self) -> int | None:
            try:
                length = int(self.headers.get("Content-Length", ""))
            except ValueError:
                length = -1
            if length < 0:
                self._empty(HTTPStatus.LENGTH_REQUIRED)
                return None
            if length > MAX_REQUEST_BYTES:
                self._empty(HTTPStatus.REQUEST_ENTITY_TOO_LARGE)
                return None
            return length

        def _json(self, status: HTTPStatus, payload: dict[str, Any]) -> None:
            body = json.dumps(payload, separators=(",", ":")).encode("utf-8")
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def _empty(self, status: HTTPStatus, allow: str | None = None) -> None:
            self.send_response(status)
            if allow is not None:
                self.send_header("Allow", allow)
            self.send_header("Content-Length", "0")
            self.end_headers()

    return DatumMcpHttpHandler


def _error(code: int, message: str) -> dict[str, Any]:
    return {"jsonrpc": "2.0", "id": None, "error": {"code": code, "message": message}}


def _valid_origin(origin: str) -> bool:
    return origin.startswith("http://") or origin.startswith("https://")
