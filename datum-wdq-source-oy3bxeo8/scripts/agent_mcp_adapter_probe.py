#!/usr/bin/env python3
"""Fake native agent client for the governed MCP adapter acceptance proof."""

from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import sys
from typing import Any


def _argument_after(arguments: list[str], flag: str) -> str:
    try:
        return arguments[arguments.index(flag) + 1]
    except (ValueError, IndexError) as exc:
        raise RuntimeError(f"missing native adapter argument {flag}") from exc


def _json_config(path: str | os.PathLike[str]) -> tuple[str, list[str]]:
    document = json.loads(Path(path).read_text(encoding="utf-8"))
    server = document["mcpServers"]["datum"]
    return server["command"], server["args"]


def _broker_command(adapter: str, arguments: list[str]) -> tuple[str, list[str]]:
    if adapter == "codex":
        overrides: dict[str, Any] = {}
        for index, argument in enumerate(arguments):
            if argument == "-c" and index + 1 < len(arguments):
                key, value = arguments[index + 1].split("=", 1)
                overrides[key] = json.loads(value)
        return overrides["mcp_servers.datum.command"], overrides["mcp_servers.datum.args"]
    if adapter == "claude-code":
        return _json_config(_argument_after(arguments, "--mcp-config"))
    if adapter == "cursor-cli":
        return _json_config(Path.cwd() / ".cursor/mcp.json")
    if adapter == "local-generic":
        return "datum-eda", [
            "mcp",
            "serve",
            "--discovery",
            os.environ["DATUM_AGENT_DISCOVERY"],
        ]
    raise RuntimeError(f"unknown adapter fixture {adapter}")


class Broker:
    def __init__(self, arguments: list[str]) -> None:
        self._next_id = 1
        self._process = subprocess.Popen(
            [os.environ["DATUM_EXPECT_CLI"], *arguments],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1,
        )

    def request(self, method: str, params: dict[str, Any] | None = None) -> dict[str, Any]:
        if self._process.stdin is None or self._process.stdout is None:
            raise RuntimeError("broker pipe is unavailable")
        identifier = self._next_id
        self._next_id += 1
        self._process.stdin.write(
            json.dumps(
                {
                    "jsonrpc": "2.0",
                    "id": identifier,
                    "method": method,
                    "params": params or {},
                },
                separators=(",", ":"),
            )
            + "\n"
        )
        self._process.stdin.flush()
        line = self._process.stdout.readline()
        if not line:
            stderr = self._process.stderr.read() if self._process.stderr else ""
            raise RuntimeError(f"broker closed before response: {stderr[:1024]}")
        response = json.loads(line)
        if response.get("id") != identifier:
            raise RuntimeError("broker response identity mismatch")
        if "error" in response:
            raise RuntimeError(f"broker protocol error: {response['error']}")
        return response["result"]

    def tool(self, name: str, arguments: dict[str, Any]) -> dict[str, Any]:
        result = self.request(
            "tools/call", {"name": name, "arguments": arguments}
        )
        return result["content"][0]["json"]

    def close(self) -> None:
        if self._process.stdin is not None:
            self._process.stdin.close()
        status = self._process.wait(timeout=5)
        if status != 0:
            stderr = self._process.stderr.read() if self._process.stderr else ""
            raise RuntimeError(f"broker exited {status}: {stderr[:1024]}")


def _cli_json(*arguments: str) -> dict[str, Any]:
    completed = subprocess.run(
        [os.environ["DATUM_EXPECT_CLI"], "--format", "json", *arguments],
        text=True,
        capture_output=True,
        check=False,
    )
    if completed.returncode != 0:
        raise RuntimeError(f"Datum CLI failed: {completed.stderr[:1024]}")
    value = json.loads(completed.stdout)
    if not isinstance(value, dict):
        raise RuntimeError("Datum CLI did not return a JSON object")
    return value


def _fence(pinned: dict[str, Any]) -> dict[str, Any]:
    return {
        "context_id": pinned["pinned_context_id"],
        "expected_model_revision": pinned["model_revision"],
        "accepted_transaction_tip": pinned.get("accepted_transaction_tip"),
    }


def _assert_ok(payload: dict[str, Any], label: str) -> None:
    if not payload.get("ok"):
        raise RuntimeError(f"{label} failed: {payload.get('error')}")


def _proposal_phase(
    broker: Broker,
    adapter: str,
    discovery: dict[str, Any],
    proposal_id: str,
) -> None:
    project = discovery["project_root"]
    pinned = json.loads(Path(discovery["pinned_context_path"]).read_text(encoding="utf-8"))
    context = broker.tool(
        "datum.context.get",
        {"session": discovery["terminal_session_id"], "project_root": project},
    )
    _assert_ok(context, "pinned context read")
    if context["result"]["context_id"] != pinned["pinned_context_id"]:
        raise RuntimeError("context read did not preserve the pinned identity")
    pinned_resource = broker.request(
        "resources/read",
        {"uri": f"datum://context/pinned/{pinned['pinned_context_id']}"},
    )
    if json.loads(pinned_resource["contents"][0]["text"])["model_revision"] != pinned["model_revision"]:
        raise RuntimeError("pinned resource revision mismatch")

    before = broker.tool("datum.query.output_jobs", {"path": project})
    _assert_ok(before, "output-job inspection")
    if before["result"]["output_job_count"] != 0:
        raise RuntimeError("fresh workflow fixture already contains an output job")
    create = broker.tool(
        "datum.proposal.create_output_job",
        {
            "path": project,
            "prefix": f"agent-{adapter}",
            "include": "bom",
            "proposal": proposal_id,
            "rationale": f"AI-WF-04 {adapter} review proof",
        }
        | _fence(pinned),
    )
    _assert_ok(create, "proposal creation")
    preview_arguments = {"path": project, "proposal": proposal_id}
    preview_a = broker.tool("datum.proposal.preview", preview_arguments)
    preview_b = broker.tool("datum.proposal.preview", preview_arguments)
    _assert_ok(preview_a, "proposal preview")
    _assert_ok(preview_b, "repeated proposal preview")
    if preview_a != preview_b:
        raise RuntimeError("proposal preview is not deterministic")
    validate = broker.tool("datum.proposal.validate", preview_arguments)
    _assert_ok(validate, "proposal validation")
    after = broker.tool("datum.query.output_jobs", {"path": project})
    _assert_ok(after, "post-proposal output-job inspection")
    if after["result"]["output_job_count"] != 0:
        raise RuntimeError("proposal preview mutated the design")

    review = _cli_json(
        "proposal",
        "review",
        project,
        "--proposal",
        proposal_id,
        "--status",
        "accepted",
    )
    if review.get("status") != "accepted":
        raise RuntimeError("owner review was not recorded")
    refreshed = broker.tool(
        "datum.context.refresh",
        {"session": discovery["terminal_session_id"], "project_root": project},
    )
    _assert_ok(refreshed, "live context refresh")
    stale = broker.tool(
        "datum.proposal.apply",
        {"path": project, "proposal": proposal_id} | _fence(pinned),
    )
    if stale.get("error", {}).get("code") != "stale_context":
        raise RuntimeError(f"stale apply was not refused: {stale}")


def _assert_native_resume(adapter: str, arguments: list[str]) -> None:
    expected = {
        "codex": ["resume", "--last"],
        "claude-code": ["--continue"],
        "cursor-cli": ["resume"],
    }.get(adapter)
    if expected is None:
        if adapter != "local-generic":
            raise RuntimeError(f"unexpected resume adapter {adapter}")
        return
    if not any(arguments[index : index + len(expected)] == expected for index in range(len(arguments))):
        raise RuntimeError(f"{adapter} did not receive its native resume arguments")


def _resume_phase(
    broker: Broker,
    adapter: str,
    arguments: list[str],
    discovery: dict[str, Any],
    proposal_id: str,
) -> None:
    _assert_native_resume(adapter, arguments)
    project = discovery["project_root"]
    pinned = json.loads(Path(discovery["pinned_context_path"]).read_text(encoding="utf-8"))
    apply = broker.tool(
        "datum.proposal.apply",
        {"path": project, "proposal": proposal_id} | _fence(pinned),
    )
    _assert_ok(apply, "approved proposal apply")
    if apply["result"].get("status") != "applied":
        raise RuntimeError("proposal did not apply")
    journal = broker.tool("datum.journal.list", {"path": project})
    _assert_ok(journal, "journal inspection")
    if journal["result"].get("transaction_count", 0) < 1:
        raise RuntimeError("authorized apply produced no journal evidence")
    refreshed = broker.tool(
        "datum.context.refresh",
        {"session": discovery["terminal_session_id"], "project_root": project},
    )
    _assert_ok(refreshed, "post-apply context refresh")
    if refreshed["result"]["model_revision"] == pinned["model_revision"]:
        raise RuntimeError("refresh did not observe the applied revision")
    live_resource = broker.request("resources/read", {"uri": "datum://context/live"})
    live = json.loads(live_resource["contents"][0]["text"])
    if live["model_revision"] != refreshed["result"]["model_revision"]:
        raise RuntimeError("live resource did not expose the refreshed revision")
    activity = broker.tool(
        "datum.context.session_activity",
        {"session": discovery["terminal_session_id"], "project_root": project, "limit": 20},
    )
    _assert_ok(activity, "resumed session activity")

    descriptor = Path(discovery["credential_descriptor"])
    authority = json.loads(descriptor.read_text(encoding="utf-8"))
    descriptor.write_text(
        json.dumps(authority | {"state": "revoked"}), encoding="utf-8"
    )
    descriptor.chmod(0o600)
    revoked = broker.tool(
        "datum.context.get",
        {"session": discovery["terminal_session_id"], "project_root": project},
    )
    if revoked.get("error", {}).get("code") != "session_authority_revoked":
        raise RuntimeError("revoked session authority did not fail closed")


def main() -> int:
    arguments = sys.argv[1:]
    adapter = _argument_after(arguments, "--probe-adapter")
    phase = _argument_after(arguments, "--probe-phase")
    if not all(os.isatty(fd) for fd in (0, 1, 2)):
        raise RuntimeError("agent fixture did not launch inside the Datum PTY")
    if "DATUM_MCP_ENDPOINT" in os.environ:
        raise RuntimeError("adapter proof must not rely on DATUM_MCP_ENDPOINT")
    command, broker_args = _broker_command(adapter, arguments)
    if command != "datum-eda":
        raise RuntimeError(f"native adapter selected unexpected MCP command {command!r}")
    expected_discovery = os.environ["DATUM_AGENT_DISCOVERY"]
    if broker_args != ["mcp", "serve", "--discovery", expected_discovery]:
        raise RuntimeError(f"native adapter selected unexpected MCP arguments {broker_args!r}")
    terminal_discovery = Path(os.environ["DATUM_DISCOVERY"])
    discovery_document = json.loads(terminal_discovery.read_text(encoding="utf-8"))
    Path(expected_discovery).write_text(
        json.dumps(discovery_document, separators=(",", ":")), encoding="utf-8"
    )
    broker = Broker(broker_args)
    capabilities = broker.request("initialize")["capabilities"]
    if not {"tools", "resources", "prompts"}.issubset(capabilities):
        raise RuntimeError("broker did not declare the required MCP capabilities")
    tools = broker.request("tools/list")["tools"]
    resources = broker.request("resources/list")["resources"]
    prompts = broker.request("prompts/list")["prompts"]
    if not any(tool["name"].startswith("datum.") for tool in tools):
        raise RuntimeError("native client could not enumerate typed Datum tools")
    if not any(
        resource["uri"] == "datum://context/live"
        for resource in resources
    ):
        raise RuntimeError("native client could not enumerate Datum resources")
    if not any(
        prompt["name"] == "datum.prepare-proposal"
        for prompt in prompts
    ):
        raise RuntimeError("native client could not enumerate Datum prompts")
    proposal_id = os.environ["DATUM_PROOF_PROPOSAL_ID"]
    if phase == "propose":
        _proposal_phase(broker, adapter, discovery_document, proposal_id)
    elif phase == "resume":
        _resume_phase(broker, adapter, arguments, discovery_document, proposal_id)
    else:
        raise RuntimeError(f"unknown workflow proof phase {phase!r}")
    broker.close()
    print(f"AGENT_WORKFLOW_OK:{adapter}:{phase}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
